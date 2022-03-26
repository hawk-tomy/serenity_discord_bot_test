mod owner;
use owner::GENERAL_GROUP;
use serenity::framework::standard::CommandGroup;

pub fn get_groups() -> [&'static CommandGroup; 1] {
    [&GENERAL_GROUP]
}
