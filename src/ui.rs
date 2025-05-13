use bevy_egui::{
    EguiContexts,
    egui::{
        self, Color32, Frame, Label, Margin, RichText, ScrollArea, SelectableLabel, Stroke,
        TextStyle, Window, vec2,
    },
};

pub fn dialogue_ui(mut contexts: EguiContexts) {
    const NAME: &str = "Old Lady Tabernacle";
    const DIALOGUE: &str = "What the hell is all the racket out there?";
    const RESPONSES: [&str; 3] = [
        "Mrs. Tabernacle, it's me, Jake. Could I please come inside?",
        "Shut up, old wretch.",
        "It's the grim reaper, your time has come.",
    ];

    Window::new("Dialogue")
        .frame(
            Frame::new()
                .fill(Color32::BLUE)
                .stroke(Stroke {
                    color: Color32::DARK_BLUE,
                    width: 10.0,
                })
                .inner_margin(Margin::same(10)),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                // speaker row
                ui.horizontal(|ui| {
                    // left side: name and dialogue
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(NAME)
                                .text_style(TextStyle::Heading)
                                .color(Color32::WHITE),
                        );

                        Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            ui.label(DIALOGUE);
                        })
                    });

                    // right: speaker image
                    let image_size = egui::vec2(100.0, 60.0);
                    ui.add_sized(image_size, Label::new("<image here>"))
                });

                // response row
                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        RESPONSES.iter().enumerate().for_each(|(i, &response)| {
                            let selected = i == 0;
                            ui.add_sized(
                                [ui.available_width(), 24.0],
                                SelectableLabel::new(selected, response),
                            );
                        })
                    })
                })
            })
        });
}
