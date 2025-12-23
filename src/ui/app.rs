use eframe::egui;
use crate::ggpk::reader::GgpkReader;
use crate::ui::tree_view::TreeView;
use crate::ui::content_view::ContentView;
use rfd::FileDialog;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FileSelection {
    GgpkOffset(u64),
    BundleFile(u64),
}

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub struct ExplorerApp {
    reader: Option<Rc<GgpkReader>>,
    tree_view: TreeView,
    pub content_view: ContentView,
    pub status_msg: String,
    pub selected_file: Option<FileSelection>,
    pub is_poe2: bool,
    pub bundle_index: Option<crate::bundles::index::Index>,
    
    // Async loading
    load_rx: Option<Receiver<Result<(GgpkReader, Option<crate::bundles::index::Index>, bool, PathBuf, String), String>>>,
    is_loading: bool,

    pub settings: crate::settings::AppSettings,
}

impl ExplorerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut content_view = ContentView::default();
        
        // Try to load schema
        let schema_path = r"s:\_projects_\_poe2_\dat-schema\schema.min.json";
        if let Ok(data) = std::fs::read(schema_path) {
             if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&data) {
                 let created_at = value.get("createdAt")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                 
                 if let Ok(s) = serde_json::from_value::<crate::dat::schema::Schema>(value) {
                     content_view.set_dat_schema(s, created_at);
                 } else {
                     println!("Failed to parse schema structure");
                 }
             } else {
                 println!("Failed to parse schema JSON");
             }
        } else {
             println!("Failed to read schema.min.json at {}", schema_path);
        }

        let settings = crate::settings::AppSettings::load();
        let mut app = Self {
            reader: None,
            tree_view: TreeView::default(),
            content_view,
            status_msg: "Ready".to_string(),
            selected_file: None,
            is_poe2: false,
            bundle_index: None,
            load_rx: None,
            is_loading: false,
            settings,
        };

        // Auto-load if path exists
        if let Some(path) = &app.settings.ggpk_path {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
               app.open_ggpk_path(p, &_cc.egui_ctx);
            }
        }

        app
    }

    fn open_ggpk(&mut self, ctx: &egui::Context) {
        if let Some(path) = FileDialog::new().add_filter("GGPK", &["ggpk"]).pick_file() {
            self.settings.ggpk_path = Some(path.to_string_lossy().to_string());
            self.settings.save();
            self.open_ggpk_path(path, ctx);
        }
    }

    fn open_ggpk_path(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.status_msg = format!("Opening {}... (This may take a moment)", path.display());
            self.is_loading = true;
            self.reader = None;
            self.bundle_index = None;
            self.tree_view = TreeView::default();
            
            let (tx, rx) = channel();
            self.load_rx = Some(rx);
            
            let path_clone = path.clone();
            let ctx_clone = ctx.clone();
            
            thread::spawn(move || {
                let start_total = std::time::Instant::now();
                let result = (|| -> Result<(GgpkReader, Option<crate::bundles::index::Index>, bool, PathBuf, String), String> {
                    let start_open = std::time::Instant::now();
                    let reader = GgpkReader::open(&path_clone)
                        .map_err(|e| format!("Failed to open GGPK: {}", e))?;
                    println!("GgpkReader::open took {:?}", start_open.elapsed());
                    
                    let mut bundle_index = None;
                    let mut extra_status = String::new();
                    let mut found_bundle_index = false;

                    // Try to load from cache first
                    let cache_path = std::path::Path::new("bundles2.cache");
                    if cache_path.exists() {
                         eprintln!("Loading index from cache...");
                         let start_cache = std::time::Instant::now();
                         match crate::bundles::index::Index::load_from_cache(cache_path) {
                             Ok(index) => {
                                 println!("Index::load_from_cache took {:?}", start_cache.elapsed());
                                 bundle_index = Some(index);
                                 extra_status = " (Cached)".to_string();
                                 found_bundle_index = true;
                                 eprintln!("Index loaded from cache successfully.");
                             },
                             Err(e) => {
                                 eprintln!("Failed to load cache: {}", e);
                             }
                         }
                    }

                    if !found_bundle_index {
                        let start_scan = std::time::Instant::now();
                        // Always try to load bundled index first
                        match reader.read_file_by_path("Bundles2/_.index.bin") {
                            Ok(Some(file_record)) => {
                                match reader.get_data_slice(file_record.data_offset, file_record.data_length) {
                                    Ok(data) => {
                                        let mut cursor = std::io::Cursor::new(data);
                                        match crate::bundles::bundle::Bundle::read_header(&mut cursor) {
                                            Ok(bundle) => {
                                                match bundle.decompress(&mut cursor) {
                                                    Ok(decompressed) => {
                                                        match crate::bundles::index::Index::read(&decompressed) {
                                                            Ok(index) => {
                                                                println!("Bundle Index parsing took {:?}", start_scan.elapsed());
                                                                // Save to cache
                                                                if let Err(e) = index.save_to_cache(cache_path) {
                                                                    println!("Failed to save cache: {}", e);
                                                                }
                                                                
                                                                bundle_index = Some(index);
                                                                extra_status = " (Bundled)".to_string();
                                                                found_bundle_index = true;
                                                            },
                                                            Err(e) => extra_status = format!(" (Index Parse Error: {})", e),
                                                        }
                                                    },
                                                    Err(e) => extra_status = format!(" (Decompress Error: {})", e),
                                                }
                                            },
                                            Err(e) => extra_status = format!(" (Bundle Header Error: {})", e),
                                        }
                                    },
                                    Err(e) => extra_status = format!(" (Read Error: {})", e),
                                }
                            },
                            Ok(None) => {}, // Not found, normal for PoE 1
                            Err(e) => extra_status = format!(" (Find Error: {})", e),
                        }
                    }
                    
                    let is_poe2 = reader.version >= 4 || found_bundle_index;
                    println!("Total Loading Thread took {:?}", start_total.elapsed());
                    
                    Ok((reader, bundle_index, is_poe2, path_clone, extra_status))
                })();
                
                let _ = tx.send(result);
                ctx_clone.request_repaint();
            });
    }

}

impl eframe::App for ExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll loader
        if self.is_loading {
             if let Some(rx) = &self.load_rx {
                 match rx.try_recv() {
                     Ok(result) => {
                         self.is_loading = false;
                         self.load_rx = None;
                         
                         match result {
                             Ok((reader, index, is_poe2, path, extra_status)) => {
                                 let reader = Rc::new(reader);
                                 self.reader = Some(reader.clone());
                                 self.bundle_index = index;
                                 self.is_poe2 = is_poe2;
                                 
                                 let start_tree = std::time::Instant::now();
                                 if let Some(idx) = &self.bundle_index {
                                     self.tree_view = TreeView::new_bundled(reader.clone(), idx);
                                 } else {
                                     self.tree_view = TreeView::new(reader.clone());
                                 }
                                 println!("TreeView::new_bundled took {:?}", start_tree.elapsed());
                                 
                                 let version = reader.version;
                                 let game_ver = if self.is_poe2 { "Target: PoE 2" } else { "Target: PoE 1" };
                                 self.status_msg = format!("Opened {:?} (v{}, {}){}", path, version, game_ver, extra_status);
                             },
                             Err(e) => {
                                 self.status_msg = format!("Error: {}", e);
                             }
                         }
                     },
                     Err(std::sync::mpsc::TryRecvError::Empty) => {},
                     Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                         self.is_loading = false;
                         self.load_rx = None; // clear it
                         self.status_msg = "Error: Loaing thread disconnected (Panic?)".to_string();
                         eprintln!("Loading thread disconnected!");
                     }
                 }
             }
        }
        

    
        // ... top panel ...
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open GGPK...").clicked() {
                        self.open_ggpk(ui.ctx());
                        ui.close_menu();
                    }
                    if ui.button("Check for Schema Update").clicked() {
                        self.content_view.dat_viewer.request_update_schema = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Export", |ui| {
                    if ui.button("Export Currently Selected File").clicked() {
                        // Logic similar to "Export File" in content view
                        // Need robust way to trigger this from menu.
                         if let Some(sel) = &self.selected_file {
                            match sel {
                                FileSelection::BundleFile(hash) => {
                                     // We need to access index, reader etc.
                                     // This is slightly awkward in `menu_button` as we need `self` which is borrowed.
                                     // But `ui.button` call is fine.
                                     // We can call a helper method, but `self` is borrowed mutably by `update`.
                                     // Which is fine? `ui` borrows from ctx?
                                     // Actually eframe update takes `&mut self`.
                                     // `ui.menu_button` takes closure `|ui|`.
                                     // Inside closure we capture `self`. 
                                     // Yes, this is fine because it's immediate mode.
                                     
                                     if let Some(reader) = &self.reader {
                                         if let Some(index) = &self.bundle_index {
                                             if let Some(file_info) = index.files.get(hash) {
                                                 self.content_view.export_bundled_content(reader, index, file_info);
                                             }
                                         }
                                     }
                                },
                                FileSelection::GgpkOffset(_) => {
                                    // TODO: GGPK export
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("Batch Export (TODO)");
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.is_loading {
                    ui.spinner();
                    ui.label("Mounting/Loading...");
                }
                ui.label(&self.status_msg);
            });
        });

        egui::SidePanel::left("tree_panel")
            .resizable(true)
            .default_width(320.0)
            .min_width(200.0)
            .show(ctx, |ui| {
             if self.reader.is_some() {
                 ui.push_id("tree_scroll", |ui| {
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                 let action = self.tree_view.show(ui, &mut self.selected_file);
                 match action {
                     crate::ui::tree_view::TreeViewAction::None => {},
                     crate::ui::tree_view::TreeViewAction::Select(_) => {}, // Handled by mut ref
                     crate::ui::tree_view::TreeViewAction::ExportBundleFolder(hashes, _folder_name) => {
                         if let Some(target_dir) = rfd::FileDialog::new().set_directory("/").pick_folder() {
                             // Clone necessary data for thread/loop
                             // Doing it on main thread for now (blocking) to ensure simplicity.
                             // Ideally spawn thread.
                             
                             if let Some(reader) = &self.reader {
                                 if let Some(index) = &self.bundle_index {
                                      // TODO: DAT -> JSON requires schema
                                      
                                      for hash in hashes {
                                          if let Some(file_info) = index.files.get(&hash) {
                                              // Read file
                                               if let Some(bundle_info) = index.bundles.get(file_info.bundle_index as usize) {
                                                 let bundle_path = format!("Bundles2/{}", bundle_info.name);
                                                 if let Ok(Some(file_record)) = reader.read_file_by_path(&bundle_path) {
                                                     if let Ok(data) = reader.get_data_slice(file_record.data_offset, file_record.data_length) {
                                                         let mut cursor = std::io::Cursor::new(data);
                                                         if let Ok(bundle) = crate::bundles::bundle::Bundle::read_header(&mut cursor) {
                                                             if let Ok(decompressed_data) = bundle.decompress(&mut cursor) {
                                                                 let start = file_info.file_offset as usize;
                                                                 let end = start + file_info.file_size as usize;
                                                                 if end <= decompressed_data.len() {
                                                                     let file_data = &decompressed_data[start..end];
                                                                     
                                                                     // Determine relative path?
                                                                     // file_info.path is full path like "Art/Textures/..."
                                                                     // folder_name is like "Textures".
                                                                     // We just write (target_dir / file_info.path).
                                                                     // We need to create parent dirs.
                                                                     
                                                                     let relative_path = std::path::Path::new(&file_info.path);
                                                                     let full_path = target_dir.join(relative_path);
                                                                     
                                                                     if let Some(parent) = full_path.parent() {
                                                                         let _ = std::fs::create_dir_all(parent);
                                                                     }
                                                                     
                                                                     // Convert?
                                                                     if file_info.path.ends_with(".dds") {
                                                                         // Convert to WebP
                                                                         // Try image crate
                                                                         let mut img_opt = None;
                                                                          if let Ok(img) = image::load_from_memory(file_data) {
                                                                              img_opt = Some(img);
                                                                          } else {
                                                                              // Try image_dds
                                                                               let mut cursor = std::io::Cursor::new(file_data);
                                                                               if let Ok(dds) = ddsfile::Dds::read(&mut cursor) {
                                                                                   if let Ok(image) = image_dds::image_from_dds(&dds, 0) {
                                                                                       img_opt = Some(image::DynamicImage::ImageRgba8(image));
                                                                                   }
                                                                               }
                                                                          }
                                                                          
                                                                          if let Some(img) = img_opt {
                                                                              let webp_path = full_path.with_extension("webp");
                                                                              // Encoding to WebP requires `image` crate with unchecked features or specific encoder.
                                                                              // `image` 0.25 supports webp.
                                                                              // save_buffer_with_format...
                                                                              // Actually `img.save_with_format(path, image::ImageFormat::WebP)`
                                                                              let _ = img.save_with_format(webp_path, image::ImageFormat::WebP);
                                                                          } else {
                                                                              // Failed to decode, write raw dds
                                                                              let _ = std::fs::write(&full_path, file_data);
                                                                          }
                                                                     } else if file_info.path.ends_with(".dat") || file_info.path.ends_with(".datc64") || file_info.path.ends_with(".datl") || file_info.path.ends_with(".datl64") { 
                                                                         // Try to export JSON if schema available match
                                                                         let mut json_exported = false;
                                                                         if let Some(json) = self.content_view.dat_viewer.convert_to_json(file_data, &file_info.path) {
                                                                             let json_path = full_path.with_extension("json");
                                                                             if std::fs::write(&json_path, json).is_ok() {
                                                                                 json_exported = true;
                                                                             }
                                                                         }
                                                                         
                                                                         if !json_exported {
                                                                             let _ = std::fs::write(&full_path, file_data);
                                                                         }
                                                                     } else {
                                                                         let _ = std::fs::write(&full_path, file_data);
                                                                     }
                                                                 }
                                                             }
                                                         }
                                                     }
                                                 }
                                               }
                                          }
                                      }
                                 }
                             }
                         }
                     }
                 }

                    });
                 });
             } else {
                 ui.label("No GGPK loaded");
             }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
             if let Some(reader) = &self.reader {
                 self.content_view.show(ui, reader, self.selected_file, self.is_poe2, &self.bundle_index);
             } else {
                 ui.centered_and_justified(|ui| {
                     if self.is_loading {
                         ui.heading("Loading GGPK...");
                         ui.spinner();
                         ui.label("Please wait while the file index is built.");
                     } else {
                         ui.label("Open a Content.ggpk file to begin.");
                     }
                 });
             }
        });
        if self.content_view.dat_viewer.request_update_schema {
             self.content_view.dat_viewer.request_update_schema = false;
             self.status_msg = "Updating Schema...".to_string();
             self.is_loading = true;
             
             // Spawn update thread
             // For now, synchronous is easier but blocks UI.
             // Let's do a quick synchronous download/copy since file is small.
             
             // Logic:
             // 1. Try to fetch from https://github.com/poe-tool-dev/dat-schema/releases/latest/download/schema.min.json
             // 2. Or if repo exists, git pull?
             
             let update_result = std::thread::spawn(|| {
                  let url = "https://github.com/poe-tool-dev/dat-schema/releases/latest/download/schema.min.json";
                  match reqwest::blocking::get(url) {
                      Ok(resp) => {
                          if resp.status().is_success() {
                              match resp.text() {
                                  Ok(text) => {
                                      // Save to local file
                                      let path = "s:/_projects_/_poe2_/dat-schema/schema.min.json";
                                      if let Err(e) = std::fs::write(path, &text) {
                                          return Err(format!("Failed to write schema: {}", e));
                                      }
                                      Ok(text)
                                  },
                                  Err(e) => Err(format!("Failed to read text: {}", e))
                              }
                          } else {
                              Err(format!("HTTP Error: {}", resp.status()))
                          }
                      },
                      Err(e) => Err(format!("Network Error: {}", e))
                  }
             }).join();
             
             self.is_loading = false;
             
              match update_result {
                  Ok(Ok(json_text)) => {
                      // Reload
                       if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_text) {
                           let created_at = value.get("createdAt")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                           if let Ok(schema) = serde_json::from_value(value) {
                               self.content_view.set_dat_schema(schema, created_at);
                               self.status_msg = "Schema Updated Successfully".to_string();
                           } else {
                               self.status_msg = "Failed to parse new schema structure".to_string();
                           }
                       } else {
                           self.status_msg = "Failed to parse new schema JSON".to_string();
                       }
                  },
                 Ok(Err(e)) => {
                      self.status_msg = format!("Update Failed: {}", e);
                 },
                 Err(_) => {
                      self.status_msg = "Update Thread Panicked".to_string();
                 }
             }
        }
    }
}
