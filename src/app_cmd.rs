mod example;
use serenity::builder::CreateApplicationCommand;
use serenity::model::{
    id::GuildId,
    interactions::{
        application_command::{
            ApplicationCommandOptionType, ApplicationCommandInteraction
        },
        Interaction, InteractionResponseType,
    },
};
use serenity::prelude::*;

use tracing::error;

pub trait ApplicationCommandTrait {
    fn get_name()-> &'static str;
    fn setup_app_cmd(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
    fn interaction_handler(ctx: &Context, interaction: &ApplicationCommandInteraction) -> String;
}

pub async fn setup_app_cmd(ctx: &Context) {
    use std::env;
    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be an integer"),
    );

    let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
        commands
            .create_application_command(example::ExampleCommand::setup_app_cmd)
            .create_application_command(|command| {
                command
                    .name("id")
                    .description("Get a user id")
                    .create_option(|option| {
                        option
                            .name("id")
                            .description("The user to lookup")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
            })
            .create_application_command(|command| {
                command
                    .name("welcome")
                    .description("Welcome a user")
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user to welcome")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("message")
                            .description("The message to send")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                            .add_string_choice(
                                "Welcome to our cool server! Ask me if you need help",
                                "pizza",
                            )
                            .add_string_choice("Hey, do you want a coffee?", "coffee")
                            .add_string_choice(
                                "Welcome to the club, you're now a good person. Well, I hope.",
                                "club",
                            )
                            .add_string_choice(
                                "I hope that you brought a controller to play together!",
                                "game",
                            )
                    })
            })
            .create_application_command(|command| {
                command
                    .name("numberinput")
                    .description("Test command for number input")
                    .create_option(|option| {
                        option
                            .name("int")
                            .description("An integer from 5 to 10")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(5)
                            .max_int_value(10)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("number")
                            .description("A float from -3.3 to 234.5")
                            .kind(ApplicationCommandOptionType::Number)
                            .min_number_value(-3.3)
                            .max_number_value(234.5)
                            .required(true)
                    })
            })
    })
    .await;
}

pub async fn interaction_handler(ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
        let content = match command.data.name.as_str() {
            "ping" => example::ExampleCommand::interaction_handler(&ctx, &command),
            name => {
                error!("You forgot this cmd {}", name);
                "This command is not found, so please report to bot dev.".to_string()
            }
        };

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
        {
            error!("cannot res to slash cmd: {}", why);
        }
    }
}
