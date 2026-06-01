use dioxus::prelude::*;

#[component]
pub fn Pagination(page: u64, pages: u64, on_page_change: EventHandler<u64>) -> Element {
    let has_prev = page > 1;
    let has_next = page < pages;

    let page_numbers = build_page_numbers(page, pages);

    rsx! {
        nav { class: "pagination", aria_label: "Pagination",
            button {
                disabled: !has_prev,
                onclick: move |_| {
                    if has_prev {
                        on_page_change.call(page - 1);
                    }
                },
                "←"
            }

            for p in page_numbers {
                if p == page {
                    span { class: "current-page", key: "{p}", "{p}" }
                } else {
                    button {
                        key: "{p}",
                        onclick: move |_| {
                            on_page_change.call(p);
                        },
                        "{p}"
                    }
                }
            }

            button {
                disabled: !has_next,
                onclick: move |_| {
                    if has_next {
                        on_page_change.call(page + 1);
                    }
                },
                "→"
            }
        }
    }
}

fn build_page_numbers(current: u64, total: u64) -> Vec<u64> {
    if total == 0 {
        return vec![1];
    }

    let mut pages = Vec::new();

    if total <= 7 {
        for i in 1..=total {
            pages.push(i);
        }
        return pages;
    }

    pages.push(1);

    let range_start = current.saturating_sub(2).max(2);
    let range_end = (current + 2).min(total - 1);

    if range_start > 2 {
        pages.push(0);
    }

    for i in range_start..=range_end {
        pages.push(i);
    }

    if range_end < total - 1 {
        pages.push(0);
    }

    pages.push(total);

    pages
}
