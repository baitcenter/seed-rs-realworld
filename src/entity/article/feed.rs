use crate::entity::{paginated_list, article, Credentials, author, timestamp, page_number};
use crate::{session, GMsg, route, request, page, logger};
use seed::prelude::*;
use std::borrow::Cow;

// Model

#[derive(Default)]
pub struct Model {
    session: session::Session,
    errors: Vec<String>,
    articles: paginated_list::PaginatedList<article::Article>,
}

// Init

pub fn init(
    session: session::Session,
    articles: paginated_list::PaginatedList<article::Article>
) -> Model {
    Model {
        session,
        articles,
        ..Model::default()
    }
}

// View

pub struct Tab<Ms> {
    title: Cow<'static, str>,
    msg: Ms,
    active: bool
}

impl<Ms> Tab<Ms> {
    pub fn new(title: impl Into<Cow<'static, str>>, msg: Ms) -> Self {
        Self {
            title: title.into(),
            msg,
            active: false
        }
    }
    pub fn activate(mut self) -> Self {
        self.active = true;
        self
    }
}

pub fn view_tabs<Ms: Clone>(tabs: Vec<Tab<Ms>>) -> Node<Ms> {
    ul![
        class!["nav", "nav-pills", "outline-active"],
        tabs.into_iter().map(view_tab)
    ]
}

fn view_tab<Ms: Clone>(tab: Tab<Ms>) -> Node<Ms> {
    li![
        class!["nav-item"],
        a![
            class!["nav-link", "active" => tab.active],
            attrs!{At::Href => ""},
            tab.title,
            simple_ev(Ev::Click, tab.msg)
        ]
    ]
}

fn view_page_link<Ms: Clone>(
    page_number: page_number::PageNumber,
    active: bool,
    msg: Ms
) -> Node<Ms> {
    li![
        class!["page-item", "active" => active],
        a![
            class!["page-link"],
            attrs!{At::Href => ""},
            simple_ev(Ev::Click, msg),
            page_number.to_string()
        ]
    ]
}

pub fn view_pagination<Ms: Clone>(
    model: &Model,
    current_page: page_number::PageNumber,
    msg_constructor: fn(page_number::PageNumber) -> Ms
) -> Node<Ms> {
    if model.articles.total_pages() > 1 {
        ul![
            class!["pagination"],
            (1..=model.articles.total_pages())
                .map(page_number::PageNumber::new)
                .map(|page_number| view_page_link(
                    page_number,
                    page_number == current_page,
                    msg_constructor(page_number)
                ))
        ]
    } else {
        empty![]
    }
}

fn view_favorite_button(credentials: Option<&Credentials>, article: &article::Article) -> Node<Msg> {
    match credentials {
        None => empty![],
        Some(_) => {
            if article.favorited {
                button![
                    class!["btn","btn-primary", "btn-sm", "pull-xs-right"],
                    simple_ev(Ev::Click, Msg::FavoriteClicked(article.slug.clone())),
                    i![
                        class!["ion-heart"],
                        format!(" {}", article.favorites_count),
                    ]
                ]
            } else {
                button![
                    class!["btn","btn-outline-primary", "btn-sm", "pull-xs-right"],
                    simple_ev(Ev::Click, Msg::UnfavoriteClicked(article.slug.clone())),
                    i![
                        class!["ion-heart"],
                        format!(" {}", article.favorites_count),
                    ]
                ]
            }
        }
    }
}

fn view_tag(tag: article::tag::Tag) -> Node<Msg> {
    li![
        class!["tag-default", "tag-pill", "tag-outline"],
        tag.to_string()
    ]
}

fn view_article_preview(credentials: Option<&Credentials>, article: &article::Article) -> Node<Msg> {
    div![
        class!["article-preview"],
        div![
            class!["article-meta"],
            a![
                attrs!{At::Href => route::Route::Profile(Cow::Borrowed(article.author.username())).to_string()},
                img![
                    attrs!{At::Src => article.author.profile().avatar.src()}
                ]
            ],
            div![
                class!["info"],
                author::view(article.author.username()),
                timestamp::view(&article.created_at)
            ],
            view_favorite_button(credentials, article)
        ],
        a![
            class!["preview-link"],
            attrs!{At::Href => route::Route::Article(article.slug.clone()).to_string()},
            h1![
                article.title
            ],
            p![
                article.description
            ],
            span![
                "Read more..."
            ],
            ul![
                class!["tag-list"],
                article.tag_list.clone().into_iter().map(view_tag)
            ]
        ]
    ]
}

pub fn view_articles(model: &Model) -> Vec<Node<Msg>> {
    let credentials = model.session.viewer().map(|viewer|&viewer.credentials);

    vec![page::view_errors(Msg::DismissErrorsClicked, model.errors.clone())]
        .into_iter()
        .chain(
        if model.articles.total == 0 {
                vec![
                    div![
                        class!["article-preview"],
                        "No articles are here... yet."
                    ]
                ]
            } else {
                model.articles.values.iter().map(|article| view_article_preview(credentials, article)).collect()
            }
        ).collect()
}

// Update

#[derive(Clone)]
pub enum Msg {
    DismissErrorsClicked,
    FavoriteClicked(article::slug::Slug),
    UnfavoriteClicked(article::slug::Slug),
    FavoriteCompleted(Result<article::Article, Vec<String>>),
}

pub fn update(
    msg: Msg,
    model: &mut Model,
    orders: &mut impl Orders<Msg, GMsg>
){
    match msg {
        Msg::DismissErrorsClicked => {
            model.errors.clear();
        },
        Msg::FavoriteClicked(slug) => {
            orders.perform_cmd(request::favorite::unfavorite(
                model.session.credentials().cloned(),
                &slug,
                Msg::FavoriteCompleted
            ));
        },
        Msg::UnfavoriteClicked(slug) => {
            orders.perform_cmd(request::favorite::favorite(
                model.session.credentials().cloned(),
                &slug,
                Msg::FavoriteCompleted
            ));
        },
        Msg::FavoriteCompleted(Ok(article)) => {
            model
                .articles
                .values
                .iter_mut()
                .find(|old_article| old_article.slug == article.slug)
                .map(|old_article| *old_article = article);
        },
        Msg::FavoriteCompleted(Err(errors)) => {
            logger::errors(errors.clone());
            model.errors = errors;
        },
    }
}