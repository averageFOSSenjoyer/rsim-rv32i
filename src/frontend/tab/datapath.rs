use crate::backend::core::ComponentType;
use crate::frontend::tab::Tab;
use crate::frontend::util::datapath_component::DatapathComponent;
use crate::frontend::util::datapath_component::DatapathComponentWidget;
use crate::frontend::util::datapath_net::{DatapathNet, DatapathNetDispalyer};
use crossbeam_channel::Receiver;
use eframe::emath::Vec2;
use egui::Color32;
use egui::epaint::{PathShape, PathStroke};
use egui::{Context, Rect, Sense, Ui};
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub type DatapathComponentMap = HashMap<ComponentType, DatapathComponent>;
pub struct Datapath {
    datapath_components_receiver: Receiver<DatapathComponentMap>,
    datapath_components: DatapathComponentMap,
}

impl Datapath {
    pub fn new(datapath_components_receiver: Receiver<DatapathComponentMap>) -> Self {
        Datapath {
            datapath_components_receiver,
            datapath_components: Default::default(),
        }
    }
}

impl Tab for Datapath {
    fn name(&self) -> &'static str {
        "✨ Datapath"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        while let Ok(datapath_components) = self.datapath_components_receiver.try_recv() {
            self.datapath_components = datapath_components;
        }

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
        let (resp, painter) = ui.allocate_painter(Vec2::new(1180.0, 680.0), Sense::hover());
        let window_pos2: [f32; 2] = resp.rect.left_top().into();

        for datapath_net in DatapathNet::iter() {
            let mut points = datapath_net.get_points();
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

        for (component_type, datapath_component) in self.datapath_components.iter() {
            let frame_offset = component_type.get_frame_offset();
            let frame_size = component_type.get_frame_size();
            let datapath_component_displayer = DatapathComponentWidget {
                datapath_component: datapath_component.clone(),
                window_pos2,
                painter: painter.clone(),
            };
            ui.put(
                Rect::from_min_size(frame_offset + window_pos2.into(), frame_size),
                datapath_component_displayer,
            );
        }
    }
}
