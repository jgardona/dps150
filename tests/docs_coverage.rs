//! Conferência da documentação de arquitetura.
//!
//! Compara as funções de produção de `src/**/*.rs` com os títulos `` ### `Nome` `` de
//! `docs/modulos/<slug>.md` e confere se os links relativos de `README.md` e de `docs/**/*.md` apontam
//! para algo que existe. Usa só a biblioteca padrão e não precisa de interface gráfica nem de porta
//! serial. O formato completo está em `specs/002-architecture-docs/contracts/doc-format.md`.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

// --- Scanner de funções ------------------------------------------------------------------------

/// Substitui por espaços o conteúdo de comentários, textos e literais de caractere, preservando as
/// quebras de linha. Assim, chaves dentro deles não bagunçam a contagem do scanner.
fn blank_literals(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match (c, next) {
            ('/', Some('/')) => {
                while i < chars.len() && chars[i] != '\n' {
                    out.push(' ');
                    i += 1;
                }
            }
            ('/', Some('*')) => {
                let mut nesting = 0;
                while i < chars.len() {
                    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                        nesting += 1;
                        out.push_str("  ");
                        i += 2;
                    } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        nesting -= 1;
                        out.push_str("  ");
                        i += 2;
                        if nesting == 0 {
                            break;
                        }
                    } else {
                        out.push(blank(chars[i]));
                        i += 1;
                    }
                }
            }
            ('"', _) => {
                out.push(' ');
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        out.push(' ');
                        i += 1;
                    }
                    if i < chars.len() {
                        out.push(blank(chars[i]));
                        i += 1;
                    }
                }
                if i < chars.len() {
                    out.push(' ');
                    i += 1;
                }
            }
            ('\'', Some('\\')) => {
                // literal de caractere com escape ('\n', '\u{1F}')
                out.push(' ');
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    out.push(' ');
                    i += 1;
                }
                if i < chars.len() {
                    out.push(' ');
                    i += 1;
                }
            }
            ('\'', Some(_)) if chars.get(i + 2) == Some(&'\'') => {
                out.push_str("   ");
                i += 3;
            }
            _ => {
                // inclui lifetimes ('a), que não fecham com aspas simples
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
enum Kind {
    Impl(String),
    Mod,
    Fn,
    Other,
}

struct Frame {
    open_depth: usize,
    kind: Kind,
}

/// Remove `pub`, `pub(crate)`, `const`, `async` e `unsafe` do começo de uma declaração.
fn strip_qualifiers(mut text: &str) -> &str {
    loop {
        let before = text;
        if let Some(rest) = text.strip_prefix("pub(") {
            text = rest.split_once(')').map_or("", |(_, r)| r).trim_start();
        }
        for q in ["pub ", "const ", "async ", "unsafe "] {
            if let Some(rest) = text.strip_prefix(q) {
                text = rest.trim_start();
            }
        }
        if text == before {
            return text;
        }
    }
}

/// Tipo de um cabeçalho `impl [<…>] [Trait for] Tipo [where …] {`, sem genéricos nem caminho.
fn impl_type(header: &str) -> String {
    let mut rest = header["impl".len()..].trim_start();
    if rest.starts_with('<') {
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in rest.char_indices() {
            match c {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = rest[end..].trim_start();
    }
    let rest = rest.split('{').next().unwrap_or(rest);
    let rest = rest.split(" where ").next().unwrap_or(rest);
    let ty = rest.split_once(" for ").map_or(rest, |(_, ty)| ty);
    let ty = ty.trim().trim_start_matches('&');
    let ty = ty.split('<').next().unwrap_or(ty);
    let ty = ty.rsplit("::").next().unwrap_or(ty).trim();
    if ty.is_empty() {
        "?".to_owned()
    } else {
        ty.to_owned()
    }
}

fn ident_after<'a>(text: &'a str, keyword: &str) -> &'a str {
    let rest = text[keyword.len()..].trim_start();
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    &rest[..end]
}

/// Lista as funções de produção de um arquivo `.rs` com o nome qualificado (`Tipo::método` ou `função`).
/// Ignora itens `#[cfg(test)]` e funções aninhadas. Devolve `Err` se as chaves ficarem desbalanceadas.
fn scan_functions(source: &str) -> Result<Vec<String>, String> {
    let cleaned = blank_literals(source);
    let mut funcs = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut depth = 0usize;
    let mut nest = 0i32; // parênteses e colchetes abertos
    let mut pending: Option<Kind> = None;
    let mut cfg_test_armed = false;
    // (profundidade antes do item, já abriu a chave?)
    let mut skipping: Option<(usize, bool)> = None;

    for (n, line) in cleaned.lines().enumerate() {
        let mut text = line.trim();

        if skipping.is_none() {
            if let Some(rest) = text.strip_prefix("#[cfg(test)]") {
                cfg_test_armed = true;
                text = rest.trim_start();
            }
            if cfg_test_armed && !text.is_empty() && !text.starts_with("#[") {
                skipping = Some((depth, false));
                cfg_test_armed = false;
            }
        }

        let header = strip_qualifiers(text);
        if header.starts_with("fn ") {
            let name = ident_after(header, "fn");
            if skipping.is_none() && !name.is_empty() {
                match frames.last().map(|f| &f.kind) {
                    None | Some(Kind::Mod) => funcs.push(name.to_owned()),
                    Some(Kind::Impl(ty)) => funcs.push(format!("{ty}::{name}")),
                    Some(Kind::Fn | Kind::Other) => {}
                }
            }
            pending = Some(Kind::Fn);
        } else if header.starts_with("impl") && header[4..].starts_with([' ', '<']) {
            pending = Some(Kind::Impl(impl_type(header)));
        } else if header.starts_with("mod ") {
            pending = Some(Kind::Mod);
        } else if ["trait ", "struct ", "enum ", "union "]
            .iter()
            .any(|k| header.starts_with(k))
        {
            pending = Some(Kind::Other);
        }

        for c in line.chars() {
            match c {
                '(' | '[' => nest += 1,
                ')' | ']' => nest -= 1,
                '{' => {
                    let kind = pending.take().unwrap_or(Kind::Other);
                    frames.push(Frame {
                        open_depth: depth,
                        kind,
                    });
                    depth += 1;
                    if let Some((_, opened)) = skipping.as_mut() {
                        *opened = true;
                    }
                }
                '}' => {
                    if depth == 0 {
                        return Err(format!(
                            "chaves desbalanceadas: '}}' sem '{{' correspondente na linha {}",
                            n + 1
                        ));
                    }
                    depth -= 1;
                    if frames.last().is_some_and(|f| f.open_depth == depth) {
                        frames.pop();
                    }
                    if skipping == Some((depth, true)) {
                        skipping = None;
                    }
                }
                ';' if nest <= 0 => {
                    pending = None;
                    if skipping == Some((depth, false)) {
                        skipping = None;
                    }
                }
                _ => {}
            }
        }
    }

    if depth != 0 {
        return Err(format!(
            "chaves desbalanceadas: {depth} '{{' sem fechar no fim do arquivo"
        ));
    }
    Ok(funcs)
}

// --- Documentos --------------------------------------------------------------------------------

fn is_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// Títulos de função (`` ### `Nome` ``) de um documento, ignorando blocos cercados.
fn doc_titles(markdown: &str) -> Vec<String> {
    let mut in_fence = false;
    let mut titles = Vec::new();
    for line in markdown.lines() {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let line = line.trim_end();
        if let Some(inner) = line
            .strip_prefix("### `")
            .and_then(|rest| rest.strip_suffix('`'))
            && !inner.is_empty()
            && !inner.contains('`')
        {
            titles.push(inner.to_owned());
        }
    }
    titles
}

/// Nome do documento de módulo para um arquivo de `src/` (`logics/app.rs` → `logics-app`).
fn slug_for(relative_to_src: &Path) -> String {
    relative_to_src
        .with_extension("")
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("-")
}

/// Destinos de links `[texto](destino)` fora de blocos cercados e de código em linha, com a linha (1-based).
fn extract_links(markdown: &str) -> Vec<(usize, String)> {
    let mut in_fence = false;
    let mut links = Vec::new();
    for (n, line) in markdown.lines().enumerate() {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        // apaga o código em linha (entre crases) para não contar links de exemplo
        let mut in_code = false;
        let visible: String = line
            .chars()
            .map(|c| {
                if c == '`' {
                    in_code = !in_code;
                    ' '
                } else if in_code {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        let mut from = 0;
        while let Some(pos) = visible[from..].find("](") {
            let start = from + pos + 2;
            let Some(len) = visible[start..].find(')') else {
                break;
            };
            let destination = visible[start..start + len].split_whitespace().next();
            if let Some(destination) = destination
                && visible[..from + pos].contains('[')
            {
                links.push((n + 1, destination.to_owned()));
            }
            from = start + len;
        }
    }
    links
}

// --- Arquivos do repositório -------------------------------------------------------------------

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Arquivos com a extensão dada, em ordem, a partir de `dir` (vazio se a pasta não existir).
fn files_with_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            found.extend(files_with_extension(&path, extension));
        } else if path.extension().is_some_and(|e| e == extension) {
            found.push(path);
        }
    }
    found
}

fn display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

// --- Testes de integração ----------------------------------------------------------------------

#[test]
fn funcoes_documentadas() {
    let root = repo_root();
    let src = root.join("src");
    let mut problems = Vec::new();
    let mut documented_slugs = BTreeSet::new();

    for file in files_with_extension(&src, "rs") {
        let relative = file.strip_prefix(&src).unwrap_or(&file);
        let source = fs::read_to_string(&file).expect("não foi possível ler o arquivo-fonte");
        let functions = match scan_functions(&source) {
            Ok(functions) => functions,
            Err(message) => {
                problems.push(format!("{}: {message}", display(&root, &file)));
                continue;
            }
        };
        if functions.is_empty() {
            continue;
        }

        let slug = slug_for(relative);
        let doc_path = root.join("docs/modulos").join(format!("{slug}.md"));
        let doc_name = display(&root, &doc_path);
        documented_slugs.insert(format!("{slug}.md"));

        let Ok(markdown) = fs::read_to_string(&doc_path) else {
            problems.push(format!(
                "documento ausente: {doc_name} ({} tem {} funções)",
                display(&root, &file),
                functions.len()
            ));
            continue;
        };

        let titles = doc_titles(&markdown);
        let in_code: BTreeSet<&String> = functions.iter().collect();
        let in_docs: BTreeSet<&String> = titles.iter().collect();
        for name in in_code.difference(&in_docs) {
            problems.push(format!("faltando em {doc_name}: {name}"));
        }
        for name in in_docs.difference(&in_code) {
            problems.push(format!("sobrando em {doc_name}: {name}"));
        }
        for name in &in_docs {
            if titles.iter().filter(|t| t == name).count() > 1 {
                problems.push(format!("título duplicado em {doc_name}: {name}"));
            }
        }
    }

    for doc in files_with_extension(&root.join("docs/modulos"), "md") {
        let name = doc
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !documented_slugs.contains(&name) {
            problems.push(format!("documento sem módulo: {}", display(&root, &doc)));
        }
    }

    problems.sort();
    assert!(
        problems.is_empty(),
        "documentação fora de sincronia com o código:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn links_relativos_existem() {
    let root = repo_root();
    let mut documents = vec![root.join("README.md")];
    documents.extend(files_with_extension(&root.join("docs"), "md"));

    let mut problems = Vec::new();
    for document in documents {
        let Ok(markdown) = fs::read_to_string(&document) else {
            continue;
        };
        let base = document.parent().unwrap_or(&root);
        for (line, destination) in extract_links(&markdown) {
            if ["http://", "https://", "mailto:", "#"]
                .iter()
                .any(|p| destination.starts_with(p))
            {
                continue;
            }
            let path = destination.split('#').next().unwrap_or("");
            if !path.is_empty() && !base.join(path).exists() {
                problems.push(format!(
                    "link quebrado em {}:{line} → {destination}",
                    display(&root, &document)
                ));
            }
        }
    }

    problems.sort();
    assert!(
        problems.is_empty(),
        "links quebrados na documentação:\n  {}",
        problems.join("\n  ")
    );
}

#[cfg(test)]
mod scanner {
    use super::*;

    fn fns(source: &str) -> Vec<String> {
        scan_functions(source).expect("o trecho deveria ser válido")
    }

    #[test]
    fn metodo_de_impl() {
        let src = "impl AppState {\n    pub fn new() -> Self {\n        Self\n    }\n}\n";
        assert_eq!(fns(src), vec!["AppState::new"]);
    }

    #[test]
    fn metodo_de_trait_usa_o_tipo_que_implementa() {
        let src = "impl Default for AppState {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n";
        assert_eq!(fns(src), vec!["AppState::default"]);
    }

    #[test]
    fn impl_com_genericos_e_caminho() {
        let src = "impl<'a> egui::Widget for ReadoutRow<'a> {\n    fn ui(self, ui: &mut egui::Ui) -> egui::Response {\n        todo!()\n    }\n}\n";
        assert_eq!(fns(src), vec!["ReadoutRow::ui"]);
    }

    #[test]
    fn funcao_livre_com_visibilidade() {
        let src = "pub(crate) fn card(ui: &mut Ui) {\n}\n\nfn privada() {}\n";
        assert_eq!(fns(src), vec!["card", "privada"]);
    }

    #[test]
    fn funcao_aninhada_e_ignorada() {
        let src = "fn externa() {\n    fn interna() {}\n    interna();\n}\n";
        assert_eq!(fns(src), vec!["externa"]);
    }

    #[test]
    fn itens_de_teste_sao_ignorados() {
        let src = "impl AppState {\n    pub fn real(&self) {}\n\n    #[cfg(test)]\n    fn so_para_teste() -> (Self, u8) {\n        todo!()\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    fn auxiliar() {}\n    #[test]\n    fn caso() {}\n}\n";
        assert_eq!(fns(src), vec!["AppState::real"]);
    }

    #[test]
    fn modulo_aninhado_so_com_constantes() {
        let src = "pub mod commands {\n    pub const X: u8 = 1;\n}\n";
        assert!(fns(src).is_empty());
    }

    #[test]
    fn chaves_em_texto_e_comentarios_nao_contam() {
        let src = "fn a() {\n    let s = \"{ sem fechar\";\n    // } comentário\n    /* { */\n    let c = '}';\n}\n";
        assert_eq!(fns(src), vec!["a"]);
    }

    #[test]
    fn chaves_desbalanceadas_sao_erro() {
        let err = scan_functions("fn a() {\n").unwrap_err();
        assert!(err.contains("chaves"), "mensagem inesperada: {err}");
    }
}

#[cfg(test)]
mod titulos {
    use super::*;

    #[test]
    fn titulo_valido() {
        assert_eq!(doc_titles("### `AppState::new`\n"), vec!["AppState::new"]);
        assert_eq!(doc_titles("### `card`   \n"), vec!["card"]);
    }

    #[test]
    fn titulo_com_texto_extra_e_ignorado() {
        assert!(doc_titles("### `card` (interna)\n### Texto `card`\n#### `card`\n").is_empty());
    }

    #[test]
    fn titulo_dentro_de_bloco_cercado_e_ignorado() {
        let md = "```markdown\n### `exemplo`\n```\n### `real`\n";
        assert_eq!(doc_titles(md), vec!["real"]);
    }

    #[test]
    fn titulo_duplicado_aparece_duas_vezes() {
        assert_eq!(doc_titles("### `a`\n### `a`\n"), vec!["a", "a"]);
    }
}

#[cfg(test)]
mod slugs {
    use super::*;

    #[test]
    fn slug_do_caminho() {
        assert_eq!(slug_for(Path::new("logics/app.rs")), "logics-app");
        assert_eq!(slug_for(Path::new("gui/mod.rs")), "gui-mod");
        assert_eq!(slug_for(Path::new("controller.rs")), "controller");
    }
}

#[cfg(test)]
mod links {
    use super::*;

    #[test]
    fn extrai_destinos_com_linha() {
        let md = "texto\nveja [a](docs/a.md) e [b](https://x.y)\n";
        assert_eq!(
            extract_links(md),
            vec![(2, "docs/a.md".to_owned()), (2, "https://x.y".to_owned())]
        );
    }

    #[test]
    fn ignora_blocos_cercados_e_codigo_em_linha() {
        let md = "```\n[a](nao/existe.md)\n```\ncódigo `[b](tambem/nao.md)` e [c](ok.md)\n";
        assert_eq!(extract_links(md), vec![(4, "ok.md".to_owned())]);
    }

    #[test]
    fn titulo_opcional_do_link_e_descartado() {
        let md = "[a](docs/a.md \"título\")\n";
        assert_eq!(extract_links(md), vec![(1, "docs/a.md".to_owned())]);
    }
}
