let s = 2;

fn paragraph(args, body) {
    s += 1;
    `<p>${s}${body}</p>
`
}

fn heading(args, body) {
    `<h${args.level}>${body}<h${args.level}/>
`
}

fn soft_break(args, body) {
    "\n"
}
