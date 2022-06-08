use common::ScenarioDetails;
use iced::{scrollable, Alignment, Column, Container, Element, Length, Rule, Scrollable, Text};

use crate::{
    message::Message,
    view::{
        scenario::ScenarioView,
        style::{AlephTheme, FontSize, Spacing},
    },
};

pub struct OverviewPage {
    scenario_views: Vec<ScenarioView>,
    scroll_state: scrollable::State,
    s: String,
}

impl OverviewPage {
    pub fn new(scenarios: Option<Vec<ScenarioDetails>>, s: String) -> Self {
        match scenarios {
            None => OverviewPage {
                scenario_views: vec![],
                scroll_state: scrollable::State::new(),
                s,
            },
            Some(mut scenarios) => {
                scenarios.sort_by_key(|s| s.ident.0.clone());
                OverviewPage {
                    scenario_views: scenarios
                        .iter()
                        .map(|s| ScenarioView::new(s.clone()))
                        .collect(),
                    scroll_state: scrollable::State::new(),
                    s: String::new(),
                }
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let scenario_list = if self.scenario_views.is_empty() {
            Self::no_scenarios(self.s.clone())
        } else {
            Self::scenario_list(&mut self.scenario_views)
        };
        let scenario_list = Scrollable::new(&mut self.scroll_state)
            .push(scenario_list)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Spacing::BIG);

        Container::new(scenario_list)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(AlephTheme)
            .into()
    }

    fn no_scenarios<'a>(s: String) -> Element<'a, Message> {
        let x = option_env!("STATS_BASE_URL").unwrap_or("env not set");
        Column::new()
            .push(Text::new(format!("{} : {}", x, s)).size(FontSize::H2))
            .padding(Spacing::BIG)
            .align_items(Alignment::Center)
            .into()
    }

    fn scenario_list(scenario_views: &mut Vec<ScenarioView>) -> Element<Message> {
        scenario_views
            .iter_mut()
            .fold(Column::new().spacing(Spacing::SMALL), |col, view| {
                col.push(view.view_in_list())
                    .push(Rule::horizontal(Spacing::SMALL))
            })
            .into()
    }
}
