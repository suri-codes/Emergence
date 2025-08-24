
Okay so there are a few things we
have to build abstraction layers around

I'd like to have a clean seperation of concerns here

Multiple crates for different things
with lots of interchangability being possible

# general ideas
- local first 

the entire application should be usable without having to connect to the internet

- semantic search for notes? using local ai model?  

- lsp so its easy to link things together


Also I like what godot does by having its lsp open on a
  specific port, which allows you to just plug and play,
  maybe we can do that for emergence stuff too?


- Bring any editor

Currently Im thinking of maybe a server / client tui
  relationship where the tui just manages helix and makes it
  open the selected text based off what is selected in the
  thingy. Once the scheme pr gets merged, its possible to
  make a helix plugin. That way you can plug emergence into
  any editor.

# crates

## emergence_zk
- [ ] Note struct
  - this depends on heavily understanding how I want to implement a zk
  - [ ] crud functionality for notes
    - [ ] File system abstracted away
  - [ ] frontmatter parser/manager to identify metadata

- [ ] parse a folder as a tree of notes
- [ ] petgraph representation



## emergence_lsp 

- [template link](https://github.com/IWANABETHATGUY/tower-lsp-boilerplate)
- [ ] given a root workspace provide lsp for various thingies
