use crate::backend::core::ComponentType;
use crate::backend::core::ComponentType::*;
use crate::backend::core::Core;
use crate::frontend::tab::Tab;
use crate::frontend::util::datapath_component::DatapathComponentDisplayer;
use crate::frontend::util::datapath_net::{DatapathNet, DatapathNetDispalyer};
use eframe::emath::Vec2;
use eframe::epaint::PathStroke;
use egui::epaint::PathShape;
use egui::Color32;
use egui::{Context, Rect, Sense, Ui};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub type DatapathComponentDisplayerMap =
    HashMap<ComponentType, Arc<Mutex<dyn DatapathComponentDisplayer>>>;
pub struct Datapath {
    datapath_component_displayers: DatapathComponentDisplayerMap,
}

impl Datapath {
    pub fn new(core: Arc<Core>) -> Self {
        let mut datapath_component_displayers =
            HashMap::<ComponentType, Arc<Mutex<dyn DatapathComponentDisplayer>>>::new();
        datapath_component_displayers.insert(Alu, core.alu.clone());
        datapath_component_displayers.insert(AluMux1, core.alu_mux1.clone());
        datapath_component_displayers.insert(AluMux2, core.alu_mux2.clone());
        datapath_component_displayers.insert(Cmp, core.cmp.clone());
        datapath_component_displayers.insert(CmpMux, core.cmp_mux.clone());
        datapath_component_displayers.insert(Ir, core.ir.clone());
        datapath_component_displayers.insert(Mar, core.mar.clone());
        datapath_component_displayers.insert(MarMux, core.mar_mux.clone());
        datapath_component_displayers.insert(MemCtl, core.mem_ctl.clone());
        datapath_component_displayers.insert(MrDR, core.mrdr.clone());
        datapath_component_displayers.insert(MwDR, core.mwdr.clone());
        datapath_component_displayers.insert(Pc, core.pc.clone());
        datapath_component_displayers.insert(PcMux, core.pc_mux.clone());
        datapath_component_displayers.insert(RegFile, core.regfile.clone());
        datapath_component_displayers.insert(RegFileMux, core.regfile_mux.clone());

        Datapath {
            datapath_component_displayers,
        }
    }
}

impl Tab for Datapath {
    fn name(&self) -> &'static str {
        "Datapath"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(800.0)
            .default_width(1200.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut Ui) {
        let mut port_highlight_nets = HashSet::new();
        let (resp, painter) = ui.allocate_painter(Vec2::new(1180.0, 680.0), Sense::hover());
        let window_pos2: [f32; 2] = resp.rect.left_top().into();

        for datapath_component_displayer in self.datapath_component_displayers.values() {
            let datapath_component_displayer = datapath_component_displayer.lock().unwrap();
            let datapath_component =
                datapath_component_displayer.get_datapath_component(&mut port_highlight_nets);
            let frame_offset = datapath_component_displayer.get_frame_offset();
            let frame_size = datapath_component_displayer.get_frame_size();
            ui.put(
                Rect::from_min_size(frame_offset + window_pos2.into(), frame_size),
                datapath_component,
            );
        }

        for datapath_net in DatapathNet::iter() {
            let mut points = datapath_net.get_points(&self.datapath_component_displayers);
            for point in points.iter_mut() {
                *point += window_pos2.into();
            }
            painter.add(PathShape {
                points,
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: PathStroke::new(2.0, Color32::GRAY),
            });
        }

        for port_highlight_net in port_highlight_nets.iter() {
            let mut points = port_highlight_net.get_points(&self.datapath_component_displayers);
            for point in points.iter_mut() {
                *point += window_pos2.into();
            }
            painter.add(PathShape {
                points,
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: PathStroke::new(2.0, Color32::LIGHT_RED),
            });
        }
    }
}
