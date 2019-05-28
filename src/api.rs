use crate::commands::{Args, Result};

/// Send a reply to the channel the message was received on.  
pub(crate) fn send_reply(args: &Args, message: &str) -> Result {
    args.msg.channel_id.say(&args.cx, message)?;
    Ok(())
}

/// Return whether or not the channel name matches `channel_name`.  
pub(crate) fn channel_name_is(args: &Args, channel_name: &str) -> bool {
    let mut is_expected_channel = false;

    // Check if the channel the message is coming from is the expected channel
    let channel = args.msg.channel(&args.cx);
    channel.map(|chan| {
        chan.guild().map(|guild_chan| {
            is_expected_channel = guild_chan.read().name() == channel_name;
        });
    });

    is_expected_channel
}
