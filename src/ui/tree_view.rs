use std::rc::Rc;
use eframe::egui;
use crate::bundles::index::Index;
use crate::ggpk::reader::GgpkReader;

pub struct TreeView {
    reader: Option<Rc<GgpkReader>>,
    bundle_root: Option<BundleNode>,
}

struct BundleNode {
    name: String,
    children: std::collections::BTreeMap<String, BundleNode>,
    file_hash: Option<u64>,
}

impl Default for TreeView {
    fn default() -> Self {
        Self { reader: None, bundle_root: None }
    }
}

pub enum TreeViewAction {
    None,
    Select(crate::ui::app::FileSelection),
    ExportBundleFolder(Vec<u64>, String),
}

impl TreeView {
    pub fn new(reader: std::rc::Rc<GgpkReader>) -> Self {
        Self { reader: Some(reader), bundle_root: None }
    }

    pub fn new_bundled(reader: std::rc::Rc<GgpkReader>, index: &Index) -> Self {
        let root = Self::build_bundle_tree(index);
        Self { reader: Some(reader), bundle_root: Some(root) }
    }

    fn build_bundle_tree(index: &Index) -> BundleNode {
        let mut root = BundleNode {
            name: "Root".to_string(),
            children: std::collections::BTreeMap::new(),
            file_hash: None,
        };

        for (hash, file) in &index.files {
            if file.path.is_empty() { continue; }
            
            let parts: Vec<&str> = file.path.split(|c| c == '/' || c == '\\').collect();
            let mut current = &mut root;
            
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    // File
                    current.children.insert(part.to_string(), BundleNode {
                        name: part.to_string(),
                        children: std::collections::BTreeMap::new(),
                        file_hash: Some(*hash),
                    });
                } else {
                    // Directory
                    current = current.children.entry(part.to_string()).or_insert_with(|| BundleNode {
                        name: part.to_string(),
                        children: std::collections::BTreeMap::new(),
                        file_hash: None,
                    });
                }
            }
        }
        root
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, selected_file: &mut Option<crate::ui::app::FileSelection>) -> TreeViewAction {
        let mut action = TreeViewAction::None;
        
        if let Some(root) = &self.bundle_root {
            self.render_bundle_node(ui, root, selected_file, &mut action);
        } else if let Some(reader) = &self.reader {
            let root_offset = reader.root_offset;
            self.render_directory(ui, reader, root_offset, "Root", selected_file);
        }
        
        action
    }

    fn render_bundle_node(&self, ui: &mut egui::Ui, node: &BundleNode, selected_file: &mut Option<crate::ui::app::FileSelection>, action: &mut TreeViewAction) {
        if let Some(hash) = node.file_hash {
            if ui.button(&node.name).clicked() {
                 *selected_file = Some(crate::ui::app::FileSelection::BundleFile(hash));
                 *action = TreeViewAction::Select(crate::ui::app::FileSelection::BundleFile(hash));
            }
        } else {
            let id = ui.make_persistent_id(&node.name).with(&node.children.len()); 
            let header = egui::CollapsingHeader::new(&node.name)
                .id_salt(id);
                
                let response = header.show(ui, |ui| {
                    let mut children: Vec<&BundleNode> = node.children.values().collect();
                    children.sort_by(|a, b| {
                        let a_is_dir = a.file_hash.is_none();
                        let b_is_dir = b.file_hash.is_none();
                        if a_is_dir != b_is_dir {
                            b_is_dir.cmp(&a_is_dir) // True (Dir) > False (File)
                        } else {
                            a.name.cmp(&b.name)
                        }
                    });

                    for child in children {
                        self.render_bundle_node(ui, child, selected_file, action);
                    }
                });
                
            response.header_response.context_menu(|ui| {
                if ui.button("Export Folder...").clicked() {
                    let mut hashes = Vec::new();
                    self.collect_hashes(node, &mut hashes);
                    *action = TreeViewAction::ExportBundleFolder(hashes, node.name.clone());
                    ui.close_menu();
                }
            });
        }
    }

    fn collect_hashes(&self, node: &BundleNode, hashes: &mut Vec<u64>) {
        if let Some(h) = node.file_hash {
            hashes.push(h);
        }
        for child in node.children.values() {
            self.collect_hashes(child, hashes);
        }
    }

    fn render_directory(&self, ui: &mut egui::Ui, reader: &GgpkReader, offset: u64, name: &str, selected_file: &mut Option<crate::ui::app::FileSelection>) {
        let id = ui.make_persistent_id(offset);
        egui::CollapsingHeader::new(name)
            .id_salt(id)
            .show(ui, |ui| {
                match reader.read_directory(offset) {
                    Ok(dir) => {
                        use crate::ggpk::record::RecordTag;
                        
                        // Collect valid entries with headers
                        let mut valid_entries = Vec::new();
                        for entry in dir.entries {
                             if let Ok(header) = reader.read_record_header(entry.offset) {
                                  valid_entries.push((entry, header));
                             }
                        }
                        
                        // Sort: PDIR first, then Name (we don't have name handy easily without reading record? Wait, file name is in file record...)
                        // PDIR name is in PDIR record.
                        // We can sort by TAG primarily. PDIR < FILE?
                        // If we want alphabetical within type, we need to read the full record.
                        // Let's settle for Type sorting first to match user request "Directories should always be first".
                        // Sorting by name within type is implicit if the directory list was already sorted?
                        // GGPK entries might be hash ordered.
                        // To sort by name, we'd need to read the names.
                        
                        // For now, let's sort by TAG: PDIR (Dir) < FILE (File).
                        // RecordTag enum usually has PDIR=... FILE=...
                        // Let's assume we want PDIR first.
                        valid_entries.sort_by(|a, b| {
                            let tag_a = a.1.tag;
                            let tag_b = b.1.tag;
                            
                            let a_is_dir = matches!(tag_a, RecordTag::PDIR);
                            let b_is_dir = matches!(tag_b, RecordTag::PDIR);
                            
                            if a_is_dir != b_is_dir {
                                b_is_dir.cmp(&a_is_dir) // True > False
                            } else {
                                // Fallback to offset if we can't read name easily without potentially expensive reads
                                a.0.offset.cmp(&b.0.offset)
                            }
                        });


                        for (entry, header) in valid_entries {
                            match header.tag {
                                RecordTag::PDIR => {
                                    match reader.read_directory(entry.offset) {
                                        Ok(sub_dir) => {
                                            self.render_directory(ui, reader, entry.offset, &sub_dir.name, selected_file);
                                        },
                                        Err(_) => { ui.label("<Read Error>"); }
                                    }
                                },
                                RecordTag::FILE => {
                                     match reader.read_file_record(entry.offset) {
                                         Ok(file) => {
                                             if ui.button(&file.name).clicked() {
                                                 *selected_file = Some(crate::ui::app::FileSelection::GgpkOffset(entry.offset));
                                             }
                                         },
                                         Err(_) => { ui.label("<Read Error>"); }
                                     }
                                },
                                _ => {}
                            }
                        }
                    },
                    Err(_) => {
                        ui.label("<Read Error>");
                    }
                }
            });
    }
}
