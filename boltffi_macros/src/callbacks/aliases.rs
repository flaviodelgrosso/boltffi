use proc_macro2::Span;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use syn::punctuated::Punctuated;
use syn::{Item, Path, PathArguments, PathSegment, Type, UseTree};

pub(crate) fn foreign_trait_path(trait_path: &Path) -> Path {
    let resolved = resolve_alias_path(trait_path).unwrap_or_else(|| trait_path.clone());
    foreign_trait_path_from(&resolved)
}

fn foreign_trait_path_from(trait_path: &Path) -> Path {
    let foreign_ident = trait_path
        .segments
        .last()
        .map(|segment| syn::Ident::new(&format!("Foreign{}", segment.ident), segment.ident.span()))
        .unwrap_or_else(|| syn::Ident::new("Foreign", Span::call_site()));
    let mut foreign_path = trait_path.clone();
    if let Some(segment) = foreign_path.segments.last_mut() {
        segment.ident = foreign_ident;
    }
    foreign_path
}

fn resolve_alias_path(trait_path: &Path) -> Option<Path> {
    let resolver = alias_resolver_for_call_site()?;
    resolver.resolve_path(trait_path)
}

fn alias_resolver_for_call_site() -> Option<AliasResolver> {
    build_alias_resolver()
}

fn build_alias_resolver() -> Option<AliasResolver> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let src_root = PathBuf::from(manifest_dir).join("src");
    let files = list_rs_files(&src_root)?;
    let resolver = files
        .iter()
        .filter_map(|file_path| {
            let content = fs::read_to_string(file_path).ok()?;
            let syntax = syn::parse_file(&content).ok()?;
            Some(AliasResolver::from_items(&syntax.items))
        })
        .fold(AliasResolver::default(), |mut acc, next| {
            acc.merge(next);
            acc
        });
    Some(resolver)
}

fn list_rs_files(root: &FsPath) -> Option<Vec<PathBuf>> {
    fs::read_dir(root)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .try_fold(Vec::new(), |mut acc, path| {
            if path.is_dir() {
                let mut nested = list_rs_files(&path)?;
                acc.append(&mut nested);
                Some(acc)
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                acc.push(path);
                Some(acc)
            } else {
                Some(acc)
            }
        })
}

#[derive(Default, Clone)]
struct AliasResolver {
    use_aliases: HashMap<String, Vec<PathSegment>>,
    type_aliases: HashMap<String, Vec<PathSegment>>,
}

impl AliasResolver {
    fn from_items(items: &[Item]) -> Self {
        let mut resolver = Self::default();

        items
            .iter()
            .filter_map(|item| match item {
                Item::Use(item_use) => Some(&item_use.tree),
                _ => None,
            })
            .for_each(|tree| resolver.collect_use_tree(Vec::new(), tree));

        items
            .iter()
            .filter_map(|item| match item {
                Item::Type(item_type) => Some(item_type),
                _ => None,
            })
            .filter_map(|item_type| {
                let target = match item_type.ty.as_ref() {
                    Type::Path(type_path) => Some(path_segments(&type_path.path)),
                    _ => None,
                }?;
                Some((item_type.ident.to_string(), target))
            })
            .for_each(|(alias, target)| {
                resolver.type_aliases.insert(alias, target);
            });

        resolver
    }

    fn merge(&mut self, other: AliasResolver) {
        self.use_aliases.extend(other.use_aliases);
        self.type_aliases.extend(other.type_aliases);
    }

    fn resolve_path(&self, path: &Path) -> Option<Path> {
        let segments = path_segments(path);
        let first = segments.first()?;
        let first_name = first.ident.to_string();
        let is_single = segments.len() == 1;

        let resolved = self
            .use_aliases
            .get(&first_name)
            .map(|prefix| {
                let rest = segments.iter().skip(1).cloned();
                prefix.iter().cloned().chain(rest).collect::<Vec<_>>()
            })
            .or_else(|| {
                is_single
                    .then(|| self.type_aliases.get(&first_name).cloned())
                    .flatten()
            })?;

        let original_args = first.arguments.clone();
        let mut adjusted = resolved;
        if is_single && let Some(last) = adjusted.last_mut() {
            last.arguments = original_args;
        }

        Some(build_path(adjusted))
    }

    fn collect_use_tree(&mut self, prefix: Vec<PathSegment>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                let mut next_prefix = prefix;
                next_prefix.push(path_segment(&path.ident));
                self.collect_use_tree(next_prefix, &path.tree);
            }
            UseTree::Name(name) => {
                let mut target = prefix;
                target.push(path_segment(&name.ident));
                self.use_aliases.insert(name.ident.to_string(), target);
            }
            UseTree::Rename(rename) => {
                let mut target = prefix;
                target.push(path_segment(&rename.ident));
                self.use_aliases.insert(rename.rename.to_string(), target);
            }
            UseTree::Group(group) => group
                .items
                .iter()
                .for_each(|item| self.collect_use_tree(prefix.clone(), item)),
            UseTree::Glob(_) => {}
        }
    }
}

fn path_segments(path: &Path) -> Vec<PathSegment> {
    path.segments.iter().cloned().collect()
}

fn path_segment(ident: &syn::Ident) -> PathSegment {
    PathSegment {
        ident: ident.clone(),
        arguments: PathArguments::None,
    }
}

fn build_path(segments: Vec<PathSegment>) -> Path {
    Path {
        leading_colon: None,
        segments: Punctuated::from_iter(segments),
    }
}
