// impl CustomType for Metadata {
//     fn build(mut builder: TypeBuilder<Self>) {
//         builder
//             .with_name("Metadata")
//             .with_get("title", |s: &mut Self| s.title.clone())
//             .with_get("draft", |s: &mut Self| s.draft)
//             .with_get("exercises", |s: &mut Self| s.exercises)
//             .with_get("code_solutions", |s: &mut Self| s.code_solutions)
//             .with_get("cell_outputs", |s: &mut Self| s.cell_outputs)
//             .with_get("interactive", |s: &mut Self| s.interactive)
//             .with_get("editable", |s: &mut Self| s.editable)
//             .with_get("hide_sidebar", |s: &mut Self| s.layout.hide_sidebar)
//             .with_get("exclude_outputs", |s: &mut Self| s.exclude_outputs.clone())
//             .with_get("user_defined", |s: &mut Self| s.user_defined.clone());
//     }
// }
