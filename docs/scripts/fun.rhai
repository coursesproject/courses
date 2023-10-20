

let visitor = Visitor();


fn visit_code_block(block) {

    //print(block.source);

    let a = Text("hej");



    return block;
}


visitor.visit_code_block = visit_code_block;

visitor.run(doc)