use crate::utils::serialize_object_id;

use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

use wither::bson::oid::ObjectId;

use super::User;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
/// The user's address
pub struct Address {
	/// The user's country
	country: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
/// The user's gender
pub enum Gender {
	Male,
	Female,
}

/// OpenID Connect email scope
#[derive(Debug, Default, Serialize, Deserialize, Apiv2Schema)]
pub struct EmailScope {
	/// End-User's preferred e-mail address. Its value MUST conform to the RFC 5322 [RFC5322] addr-spec syntax
	pub email: String,
	/// If the user email has been verified
	pub email_verified: bool,
}

/// Available User info
/// TODO: Conform to OpenID Connect scopes
///
/// * email -> This scope value requests access to the following claims:
/// 	* email
/// 	* email_verified
/// * profile -> This scope value requests access to the End-User's default profile claims:
/// 	* name,
/// 	* family_name,
/// 	* given_name,
/// 	* middle_name,
/// 	* nickname,
/// 	* preferred_username,
/// 	* profile,
///		* picture,
///		* website,
///		* gender,
///		* birthdate,
///		* zoneinfo,
///		* locale,
///		* updated_at.
/// * address -> This scope value requests access to the following claims:
/// 	* address
/// * phone -> This scope value requests access to the following claims:  
/// 	* phone_number,
/// 	* phone_number_verified.
#[derive(Debug, Default, Serialize, Deserialize, Apiv2Schema)]
pub struct UserInfo {
	/// The ID of the model and the Subject: Identifier for the End-User at the Issuer.
	#[serde(
		rename = "_id",
		skip_serializing_if = "Option::is_none",
		serialize_with = "serialize_object_id"
	)]
	pub id: Option<ObjectId>,
	/// Email scope
	#[serde(flatten, skip_serializing_if = "Option::is_none")]
	pub email_scope: Option<EmailScope>,
	/// Shorthand name by which the End-User wishes to be referred to at the RP, such as janedoe or j.doe. This value MAY be any valid JSON string including special characters such as @, /, or whitespace.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub preferred_username: Option<String>,
	/// Given name(s) or first name(s) of the End-User. Note that in some cultures, people can have multiple given names; all can be present, with the names being separated by space characters.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub given_name: Option<String>,
	/// Middle name(s) of the End-User. Note that in some cultures, people can have multiple middle names; all can be present, with the names being separated by space characters. Also note that in some cultures, middle names are not used.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub middle_name: Option<String>,
	/// Surname(s) or last name(s) of the End-User. Note that in some cultures, people can have multiple family names or no family name; all can be present, with the names being separated by space characters.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub family_name: Option<String>,
	/// Casual name of the End-User that may or may not be the same as the given_name. For instance, a nickname value of Mike might be returned alongside a given_name value of Michael.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nickname: Option<String>,
	/// URL of the End-User's profile page. The contents of this Web page SHOULD be about the End-User.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub profile: Option<String>,
	/// URL of the End-User's profile picture. This URL MUST refer to an image file (for example, a PNG, JPEG, or GIF image file), rather than to a Web page containing an image. Note that this URL SHOULD specifically reference a profile photo of the End-User suitable for displaying when describing the End-User, rather than an arbitrary photo taken by the End-User.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub picture: Option<String>,
	/// URL of the End-User's Web page or blog. This Web page SHOULD contain information published by the End-User or an organization that the End-User is affiliated with.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub website: Option<String>,
	/// End-User's gender. Values defined by this specification are female and male. Other values MAY be used when neither of the defined values are applicable.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gender: Option<Gender>,
	/// End-User's birthday, represented as an ISO 8601:2004 [ISO8601‑2004] YYYY-MM-DD format.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub birthdate: Option<String>,
	/// String from zoneinfo [zoneinfo] time zone database representing the End-User's time zone. For example, Europe/Paris or America/Los_Angeles.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub zoneinfo: Option<String>,
	/// End-User's locale, represented as a BCP47 [RFC5646] language tag. This is typically an ISO 639-1 Alpha-2 [ISO639‑1] language code in lowercase and an ISO 3166-1 Alpha-2 [ISO3166‑1] country code in uppercase, separated by a dash. For example, en-US or fr-CA. As a compatibility note, some implementations have used an underscore as the separator rather than a dash, for example, en_US; Relying Parties MAY choose to accept this locale syntax as well.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub locale: Option<String>,
	/// End-User's preferred telephone number. E.164 [E.164] is RECOMMENDED as the format of this Claim, for example, +1 (425) 555-1212 or +56 (2) 687 2400. If the phone number contains an extension, it is RECOMMENDED that the extension be represented using the RFC 3966 [RFC3966] extension syntax, for example, +1 (604) 555-1234;ext=5678.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub phone_number: Option<String>,
	/// True if the End-User's phone number has been verified; otherwise false. When this Claim Value is true, this means that the OP took affirmative steps to ensure that this phone number was controlled by the End-User at the time the verification was performed. The means by which a phone number is verified is context-specific, and dependent upon the trust framework or contractual agreements within which the parties are operating. When true, the phone_number Claim MUST be in E.164 format and any extensions MUST be represented in RFC 3966 format.
	pub phone_number_verified: bool,
	/// End-User's preferred postal address. The value of the address member is a JSON [RFC4627] structure containing some or all of the members defined in Section 5.1.1.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub address: Option<Address>,
}

impl From<User> for UserInfo {
	fn from(user: User) -> Self {
		let User {
			id,
			preferred_username,
			email_scope,
			given_name,
			middle_name,
			family_name,
			nickname,
			profile,
			picture,
			website,
			gender,
			birthdate,
			zoneinfo,
			locale,
			phone_number,
			phone_number_verified,
			address,
			..
		} = user;

		UserInfo {
			id,
			preferred_username,
			email_scope: Some(email_scope),
			given_name,
			middle_name,
			family_name,
			nickname,
			profile,
			picture,
			website,
			gender,
			birthdate,
			zoneinfo,
			locale,
			phone_number,
			phone_number_verified,
			address,
		}
	}
}
