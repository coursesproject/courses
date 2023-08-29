
fn visit_code_block() {
    //source = get_url("https://google.com").status;
}

fn visit_inline() {
    switch inline.type {
        "Link" => {
            print(inline.value[1]);
            let response = get_url(inline.value[1]);
            if response.status != "200 OK" {
                print(`invalid link at ${inline.value[1]}. Status code ${response.status}`);
            }
        },
        _ => {}
    }
}