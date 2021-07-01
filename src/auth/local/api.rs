use lettre::{SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use log::{info, error};

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
	let email = EmailBuilder::new()
		// Destination address/alias
		.to((user_email, username))
		// Sender address/alias
		.from((address, alias))
		// Email subject
		.subject(email_title)
		// Email html body
		.html(html_mail)
		.build()?;

		// Create transport and send email
	SmtpTransport::new(SMTP_CLIENT.clone())
		.send(email.into())
		.map_err(|e| {
			error!("{:?}", e);
			AuthErrors::SendEmailError
		})?;

	Ok(())
}
