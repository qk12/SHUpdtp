use lettre::transport::smtp::{authentication::Credentials, response::Response, Error};
use lettre::{Message, SmtpTransport, Transport};

pub fn send_email_token(email: String, token: String) -> Result<Response, Error> {
    // let from = "noreply@admin.shu.edu.cn".to_string();
    let from = "1987258436@qq.com".to_string();
    let to = email.clone();
    let subject = "【SHUOJ】您的邮箱验证码".to_string();

    let contents = format!(
        "同学，你好：\n您的验证码是： {}\n\n验证码 2 小时内有效\n为了您的账号安全，请勿将验证码泄露给其他任何人!",
       token
    );

    let smtp_username = "1987258436@qq.com".to_string();
    let smtp_password = "wojkjdfglmtkcifj".to_string();
    let smtp_server = "smtp.qq.com".to_string();

    let message = Message::builder()
        .from(from.parse().unwrap())
        //.reply_to("example@example.com>".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(contents)
        .unwrap();

    let creds = Credentials::new(smtp_username, smtp_password);

    // Open a remote connection to SMTP server
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&message)
}
