
let errors = [];

let hej = 2;


fn visit_inline(inline) {
    if inline.type == "Link" {
        try {
            let response = get_url(inline.value[1]);
            if response.status != "200 OK" {
                errors += `invalid link at ${inline.value[1]}. Status code ${response.status}`;
            }
        } catch (err) {
            errors += "Other error"
        }
        
    }
    return inline;
}

fn finalize(meta) {
    if len(errors) > 0 {
        for e in errors {
            print(`Error in "${meta.title}": ${e}`)
        }
    }
    
}