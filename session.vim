let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
let NetrwMenuPriority =  80 
let NetrwTopLvlMenu = "Netrw."
silent only
silent tabonly
cd ~/caido/parseplz/http-plz
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +1 src/lib.rs
badd +10 src/message/impl_decompress.rs
badd +44 src/one/impl_decompress.rs
badd +8 src/message/mod.rs
badd +27 src/one/mod.rs
badd +44 src/one/impl_try_from_bytes.rs
badd +63 src/one/request.rs
badd +6 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/info_line/two/request.rs
badd +60 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/info_line/one/request.rs
badd +13 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/uri/mod.rs
badd +506 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/uri/path.rs
badd +29 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/header_map/mod.rs
badd +48 src/one/response.rs
badd +77 src/message/request.rs
badd +1 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/info_line/one/response.rs
badd +1 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/status.rs
badd +62 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/mod.rs
badd +82 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/methods.rs
badd +61 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/uri/builder.rs
badd +8 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/body-plz-0.0.34/src/variants/mod.rs
badd +11 src/message/response.rs
badd +3 ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/info_line/two/response.rs
badd +0 ~/caido/parseplz/http-plz/src/message/builder.rs
argglobal
%argdel
edit src/message/response.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 93 + 93) / 187)
exe 'vert 2resize ' . ((&columns * 93 + 93) / 187)
argglobal
balt ~/.cargo/git/checkouts/primitives-27691c725ff37a54/77ee83e/header-plz/src/message_head/info_line/one/response.rs
let s:cpo_save=&cpo
set cpo&vim
nmap <buffer>  hp <Plug>(GitGutterPreviewHunk)
nmap <buffer>  hu <Plug>(GitGutterUndoHunk)
nmap <buffer>  hs <Plug>(GitGutterStageHunk)
xmap <buffer>  hs <Plug>(GitGutterStageHunk)
nmap <buffer> [c <Plug>(GitGutterPrevHunk)
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
nmap <buffer> ]c <Plug>(GitGutterNextHunk)
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
xmap <buffer> ac <Plug>(GitGutterTextObjectOuterVisual)
omap <buffer> ac <Plug>(GitGutterTextObjectOuterPending)
xmap <buffer> ic <Plug>(GitGutterTextObjectInnerVisual)
omap <buffer> ic <Plug>(GitGutterTextObjectInnerPending)
let &cpo=s:cpo_save
unlet s:cpo_save
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=list:-1
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal cinoptions=L0,(s,Ws,J1,j1,m1
setlocal cinscopedecls=public,protected,private
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern,macro
setlocal colorcolumn=80
setlocal comments=s0:/*!,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//%s
setlocal complete=.,w,b,u,t
setlocal completefunc=
setlocal completeslash=
setlocal concealcursor=
setlocal conceallevel=0
setlocal nocopyindent
setlocal nocursorbind
setlocal nocursorcolumn
setlocal cursorline
setlocal cursorlineopt=both
setlocal nodiff
setlocal errorformat=%-G,%-Gerror:\ aborting\ %.%#,%-Gerror:\ Could\ not\ compile\ %.%#,%Eerror:\ %m,%Eerror[E%n]:\ %m,%Wwarning:\ %m,%Inote:\ %m,%C\ %#-->\ %f:%l:%c,%E\ \ left:%m,%C\ right:%m\ %f:%l:%c,%Z,%f:%l:%c:\ %t%*[^:]:\ %m,%f:%l:%c:\ %*\\d:%*\\d\ %t%*[^:]:\ %m,%-G%f:%l\ %s,%-G%*[\ ]^,%-G%*[\ ]^%*[~],%-G%*[\ ]...,%-G%\\s%#Downloading%.%#,%-G%\\s%#Checking%.%#,%-G%\\s%#Compiling%.%#,%-G%\\s%#Finished%.%#,%-G%\\s%#error:\ Could\ not\ compile\ %.%#,%-G%\\s%#To\ learn\ more\\,%.%#,%-G%\\s%#For\ more\ information\ about\ this\ error\\,%.%#,%-Gnote:\ Run\ with\ `RUST_BACKTRACE=%.%#,%.%#panicked\ at\ \\'%m\\'\\,\ %f:%l:%c
setlocal eventignorewin=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=79
setlocal foldmarker={{{,}}}
setlocal foldmethod=indent
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal formatoptions=croqnlj
setlocal iminsert=0
setlocal imsearch=-1
setlocal include=\\v^\\s*(pub\\s+)?use\\s+\\zs(\\f|:)+
setlocal includeexpr=rust#IncludeExpr(v:fname)
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal nolinebreak
setlocal nolisp
setlocal lispoptions=
setlocal list
setlocal makeprg=cargo\ $*
setlocal matchpairs=(:),{:},[:],<:>
setlocal nomodeline
setlocal modifiable
setlocal nrformats=bin,hex
setlocal number
setlocal numberwidth=4
setlocal omnifunc=v:lua.vim.lsp.omnifunc
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
setlocal relativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal scrollback=-1
setlocal noscrollbind
setlocal shiftwidth=4
setlocal signcolumn=yes
setlocal smartindent
setlocal nosmoothscroll
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\\t\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal spelloptions=noplainbuffer
setlocal statuscolumn=
setlocal suffixesadd=.rs
setlocal noswapfile
setlocal synmaxcol=3000
if &syntax != ''
setlocal syntax=
endif
setlocal tabstop=8
setlocal tagfunc=v:lua.vim.lsp.tagfunc
setlocal textwidth=99
setlocal noundofile
setlocal varsofttabstop=
setlocal vartabstop=
setlocal winblend=0
setlocal nowinfixbuf
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal winhighlight=
setlocal wrap
setlocal wrapmargin=0
14
sil! normal! zo
41
sil! normal! zo
54
sil! normal! zo
61
sil! normal! zo
62
sil! normal! zo
62
sil! normal! zo
let s:l = 73 - ((39 * winheight(0) + 22) / 44)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 73
normal! 042|
wincmd w
argglobal
if bufexists(fnamemodify("~/caido/parseplz/http-plz/src/message/builder.rs", ":p")) | buffer ~/caido/parseplz/http-plz/src/message/builder.rs | else | edit ~/caido/parseplz/http-plz/src/message/builder.rs | endif
if &buftype ==# 'terminal'
  silent file ~/caido/parseplz/http-plz/src/message/builder.rs
endif
balt src/message/response.rs
let s:cpo_save=&cpo
set cpo&vim
nmap <buffer>  hp <Plug>(GitGutterPreviewHunk)
nmap <buffer>  hu <Plug>(GitGutterUndoHunk)
nmap <buffer>  hs <Plug>(GitGutterStageHunk)
xmap <buffer>  hs <Plug>(GitGutterStageHunk)
nmap <buffer> [c <Plug>(GitGutterPrevHunk)
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
nmap <buffer> ]c <Plug>(GitGutterNextHunk)
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
xmap <buffer> ac <Plug>(GitGutterTextObjectOuterVisual)
omap <buffer> ac <Plug>(GitGutterTextObjectOuterPending)
xmap <buffer> ic <Plug>(GitGutterTextObjectInnerVisual)
omap <buffer> ic <Plug>(GitGutterTextObjectInnerPending)
let &cpo=s:cpo_save
unlet s:cpo_save
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=list:-1
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal cinoptions=L0,(s,Ws,J1,j1,m1
setlocal cinscopedecls=public,protected,private
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern,macro
setlocal colorcolumn=80
setlocal comments=s0:/*!,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//%s
setlocal complete=.,w,b,u,t
setlocal completefunc=
setlocal completeslash=
setlocal concealcursor=
setlocal conceallevel=0
setlocal nocopyindent
setlocal nocursorbind
setlocal nocursorcolumn
setlocal cursorline
setlocal cursorlineopt=both
setlocal nodiff
setlocal errorformat=%-G,%-Gerror:\ aborting\ %.%#,%-Gerror:\ Could\ not\ compile\ %.%#,%Eerror:\ %m,%Eerror[E%n]:\ %m,%Wwarning:\ %m,%Inote:\ %m,%C\ %#-->\ %f:%l:%c,%E\ \ left:%m,%C\ right:%m\ %f:%l:%c,%Z,%f:%l:%c:\ %t%*[^:]:\ %m,%f:%l:%c:\ %*\\d:%*\\d\ %t%*[^:]:\ %m,%-G%f:%l\ %s,%-G%*[\ ]^,%-G%*[\ ]^%*[~],%-G%*[\ ]...,%-G%\\s%#Downloading%.%#,%-G%\\s%#Checking%.%#,%-G%\\s%#Compiling%.%#,%-G%\\s%#Finished%.%#,%-G%\\s%#error:\ Could\ not\ compile\ %.%#,%-G%\\s%#To\ learn\ more\\,%.%#,%-G%\\s%#For\ more\ information\ about\ this\ error\\,%.%#,%-Gnote:\ Run\ with\ `RUST_BACKTRACE=%.%#,%.%#panicked\ at\ \\'%m\\'\\,\ %f:%l:%c
setlocal eventignorewin=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=79
setlocal foldmarker={{{,}}}
setlocal foldmethod=indent
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal formatoptions=croqnlj
setlocal iminsert=0
setlocal imsearch=-1
setlocal include=\\v^\\s*(pub\\s+)?use\\s+\\zs(\\f|:)+
setlocal includeexpr=rust#IncludeExpr(v:fname)
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0],0(,0)
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal nolinebreak
setlocal nolisp
setlocal lispoptions=
setlocal list
setlocal makeprg=cargo\ $*
setlocal matchpairs=(:),{:},[:],<:>
setlocal nomodeline
setlocal modifiable
setlocal nrformats=bin,hex
setlocal number
setlocal numberwidth=4
setlocal omnifunc=v:lua.vim.lsp.omnifunc
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
setlocal relativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal scrollback=-1
setlocal noscrollbind
setlocal shiftwidth=4
setlocal signcolumn=yes
setlocal smartindent
setlocal nosmoothscroll
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\\t\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal spelloptions=noplainbuffer
setlocal statuscolumn=
setlocal suffixesadd=.rs
setlocal noswapfile
setlocal synmaxcol=3000
if &syntax != ''
setlocal syntax=
endif
setlocal tabstop=8
setlocal tagfunc=v:lua.vim.lsp.tagfunc
setlocal textwidth=99
setlocal noundofile
setlocal varsofttabstop=
setlocal vartabstop=
setlocal winblend=0
setlocal nowinfixbuf
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal winhighlight=
setlocal wrap
setlocal wrapmargin=0
let s:l = 5 - ((4 * winheight(0) + 22) / 44)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 5
normal! 012|
wincmd w
exe 'vert 1resize ' . ((&columns * 93 + 93) / 187)
exe 'vert 2resize ' . ((&columns * 93 + 93) / 187)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
