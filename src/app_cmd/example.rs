use super::ApplicationCommandTrait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

pub struct ExampleCommand;

//TODO: remake to use proc macro
impl ApplicationCommandTrait for ExampleCommand {
    fn get_name() -> &'static str {
        "ping"
    }
    fn setup_app_cmd(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        cmd.name("ping").description("A ping command")
    }

    fn interaction_handler(_ctx: &Context, _interaction: &ApplicationCommandInteraction) -> String {
        "Hi".to_string()
    }
}
