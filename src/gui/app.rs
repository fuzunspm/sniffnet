//! Module defining the application structure: messages, updates, subscriptions.
//!
//! It also is a wrapper of gui's main two pages: initial and run page.

use std::time::Duration;

use iced::widget::Column;
use iced::{executor, Application, Command, Element, Subscription, Theme};

use crate::gui::components::footer::footer;
use crate::gui::components::header::header;
use crate::gui::components::modal::{get_clear_all_overlay, get_exit_overlay, Modal};
use crate::gui::components::types::my_modal::MyModal;
use crate::gui::pages::connection_details_page::connection_details_page;
use crate::gui::pages::initial_page::initial_page;
use crate::gui::pages::inspect_page::inspect_page;
use crate::gui::pages::notifications_page::notifications_page;
use crate::gui::pages::overview_page::overview_page;
use crate::gui::pages::settings_language_page::settings_language_page;
use crate::gui::pages::settings_notifications_page::settings_notifications_page;
use crate::gui::pages::settings_style_page::settings_style_page;
use crate::gui::pages::types::running_page::RunningPage;
use crate::gui::pages::types::settings_page::SettingsPage;
use crate::gui::styles::style_constants::get_font;
use crate::gui::types::message::Message;
use crate::gui::types::sniffer::Sniffer;
use crate::gui::types::status::Status;

/// Update period (milliseconds)
pub const PERIOD_TICK: u64 = 1000;

impl Application for Sniffer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Sniffer;

    fn new(flags: Sniffer) -> (Sniffer, Command<Message>) {
        (flags, iced::window::maximize(true))
    }

    fn title(&self) -> String {
        String::from("Sniffnet")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.update(message)
    }

    fn view(&self) -> Element<Message> {
        let status = *self.status_pair.0.lock().unwrap();
        let style = self.style;
        let font = get_font(style);

        let header = match status {
            Status::Init => header(style, false, self.language, self.last_opened_setting),
            Status::Running => header(style, true, self.language, self.last_opened_setting),
        };

        let body = match status {
            Status::Init => initial_page(self),
            Status::Running => match self.running_page {
                RunningPage::Overview => overview_page(self),
                RunningPage::Inspect => inspect_page(self),
                RunningPage::Notifications => notifications_page(self),
            },
        };

        let footer = footer(self.language, style, &self.newer_release_available.clone());

        let content = Column::new().push(header).push(body).push(footer);

        if self.modal.is_none() && self.settings_page.is_none() {
            content.into()
        } else if self.modal.is_some() {
            let overlay = match self.modal.unwrap() {
                MyModal::Quit => get_exit_overlay(style, font, self.language),
                MyModal::ClearAll => get_clear_all_overlay(style, font, self.language),
                MyModal::ConnectionDetails(connection_index) => {
                    connection_details_page(self, connection_index)
                }
            };

            Modal::new(content, overlay)
                .on_blur(Message::HideModal)
                .into()
        } else {
            let overlay = match self.settings_page.unwrap() {
                SettingsPage::Notifications => settings_notifications_page(self),
                SettingsPage::Appearance => settings_style_page(self),
                SettingsPage::Language => settings_language_page(self),
            };

            Modal::new(content, overlay)
                .on_blur(Message::CloseSettings)
                .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        const NO_MODIFIER: iced_native::keyboard::Modifiers =
            iced_native::keyboard::Modifiers::empty();
        let hot_keys_subscription =
            iced_native::subscription::events_with(|event, _| match event {
                // ctrl+Q => exit
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Q,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::Quit),
                // return => return key pressed
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Enter,
                    ..
                }) => Some(Message::ReturnKeyPressed),
                // esc => esc key pressed
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Escape,
                    ..
                }) => Some(Message::EscKeyPressed),
                // tab => switch to next page
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Tab,
                    modifiers: NO_MODIFIER,
                }) => Some(Message::SwitchPage(true)),
                // shift+tab => switch to previous page
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Tab,
                    modifiers: iced_native::keyboard::Modifiers::SHIFT,
                }) => Some(Message::SwitchPage(false)),
                // ctrl+O => open full report
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::O,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::OpenReport),
                // ctrl+, => open settings
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Comma,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::OpenLastSettings),
                // backspace => reset button pressed
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Backspace,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::ResetButtonPressed),
                // ctrl+D => ctrl+D keys pressed
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::D,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::CtrlDPressed),
                // left arrow => one page before the current one
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Left,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::ArrowPressed(false)),
                // right arrow => one page after the current one
                iced_native::Event::Keyboard(iced_native::keyboard::Event::KeyPressed {
                    key_code: iced_native::keyboard::KeyCode::Right,
                    modifiers: iced_native::keyboard::Modifiers::COMMAND,
                }) => Some(Message::ArrowPressed(true)),
                _ => None,
            });
        let time_subscription = match *self.status_pair.0.lock().unwrap() {
            Status::Running => {
                iced::time::every(Duration::from_millis(PERIOD_TICK)).map(|_| Message::TickRun)
            }
            Status::Init => {
                iced::time::every(Duration::from_millis(PERIOD_TICK)).map(|_| Message::TickInit)
            }
        };
        Subscription::batch([hot_keys_subscription, time_subscription])
    }
}
