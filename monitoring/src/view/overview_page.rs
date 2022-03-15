use iced::{scrollable, Alignment, Column, Container, Element, Length, Scrollable, Text};

use crate::{
    data::Scenario,
    message::Message,
    view::{
        scenario::ScenarioView,
        style::{AlephTheme, FontSize, Spacing},
    },
};

pub struct OverviewPage {
    scenario_views: Vec<ScenarioView>,
    scroll_state: scrollable::State,
}

impl OverviewPage {
    pub fn new(scenarios: Option<Vec<Scenario>>) -> Self {
        match scenarios {
            None => OverviewPage {
                scenario_views: vec![],
                scroll_state: scrollable::State::new(),
            },
            Some(scenarios) => OverviewPage {
                scenario_views: scenarios
                    .iter()
                    .map(|s| ScenarioView::new(s.clone()))
                    .collect(),
                scroll_state: scrollable::State::new(),
            },
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        if self.scenario_views.is_empty() {
            Self::no_scenarios()
        } else {
            let scenario_list = Self::scenario_list(&mut self.scenario_views);
            let content = Container::new(scenario_list)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .style(AlephTheme);
            content.into()
        }
    }

    fn no_scenarios<'a>() -> Element<'a, Message> {
        Column::new()
            .push(Text::new("No scenarios available").size(FontSize::H2))
            .align_items(Alignment::Center)
            .into()
    }

    fn scenario_list(scenario_views: &mut Vec<ScenarioView>) -> Element<Message> {
        scenario_views
            .iter_mut()
            .fold(Column::new().spacing(Spacing::NORMAL), |col, view| {
                col.push(view.view_in_list())
            })
            .into()
    }
}
