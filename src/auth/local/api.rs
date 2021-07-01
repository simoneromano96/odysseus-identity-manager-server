use lettre::{message::MultiPart, Message, SmtpTransport, Transport};
use log::{error, info};

use crate::{
	auth::AuthErrors,
	settings::{SMTPSettings, APP_SETTINGS, SMTP_CLIENT},
};

pub fn send_email_to_user(
	user_email: &str,
	username: &str,
	email_title: &str,
	html_mail: &str,
) -> Result<(), AuthErrors> {
	// Destructure SMTP settings
	let SMTPSettings { address, alias, .. } = &APP_SETTINGS.smtp;

	info!("Sending email to user {:?}", &username);
	info!("{:?}", &html_mail);

	// Build email
	let email = Message::builder()
		// Sender address/alias
		.from(format!("{} <{}>", alias, address).parse()?)
		// Destination address/alias
		.to(format!("{} <{}>", username, user_email).parse()?)
		// Email subject
		.subject(email_title)
		// Email html body
		.multipart(MultiPart::alternative_plain_html(
			email_title.to_string(),
			html_mail.to_string(),
		))?;

	// Create transport and send email
	SMTP_CLIENT
		.send(&email)
		.map_err(|e| {
			error!("{:?}", e);
			AuthErrors::SendEmailError
		})?;

	Ok(())
}
