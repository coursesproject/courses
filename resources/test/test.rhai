
fn repeat_n(n, body) {
    for i in 0..n {
        write(body);
    }
}


/// Msg
fn message(body) {
    write(`
    <article class="message">
        ${render(body)}
    </article>
    `)
}