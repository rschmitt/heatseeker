" Autoloaded heatseeker functions

function! s:HsContinuation(selections, first_cmd, rest_cmd, ...) abort
    if empty(a:selections)
        return
    endif
    " Close the terminal buffer if we’re still in it (best-effort)
    try | if &buftype ==# 'terminal' | bd! | endif | catch | endtry

    let l:empty_tab = (a:0 >= 1 ? a:1 : s:IsCurrentTabEmpty())
    let l:first_done = 0
    let l:use_first_for_first = (len(a:selections) ==# 1) || l:empty_tab
    for s in a:selections
        if s ==# ''
            continue
        endif
        if !l:first_done && l:use_first_for_first
            let l:first_done = 1
            execute a:first_cmd . ' ' . fnameescape(s)
        else
            let l:first_done = 1
            execute a:rest_cmd . ' ' . fnameescape(s)
        endif
    endfor
endfunction

function! s:IsCurrentTabEmpty() abort
    if winnr('$') !=# 1
        return 0
    endif
    if &buftype !=# '' || &modified
        return 0
    endif
    if bufname('%') !=# ''
        return 0
    endif
    if line('$') !=# 1 || getline(1) !=# ''
        return 0
    endif
    return 1
endfunction

function! s:HsNvimOnExit(cb, job_id, code, event) abort
    let l:sels = filereadable(a:cb.tempfile) ? readfile(a:cb.tempfile) : []
    call delete(a:cb.tempfile)
    " Close terminal buffer window best-effort, then jump back
    if win_gotoid(a:cb.term_win)
        try | execute 'bd!' a:cb.bufnr | catch | endtry
    endif
    call win_gotoid(get(a:cb, 'return_win', -1))
    call s:HsContinuation(l:sels, a:cb.first, a:cb.rest, get(a:cb, 'empty_tab', 0))
endfunction

let s:default_choice_cmd = ''
function! s:DefaultChoiceCommand() abort
    if exists('g:heatseeker_choice_command') && !empty(g:heatseeker_choice_command)
        return g:heatseeker_choice_command
    endif
    if s:default_choice_cmd !=# ''
        return s:default_choice_cmd
    endif
    if executable('fd')
        let s:default_choice_cmd = 'fd --type f --color=never'
    elseif executable('rg')
        let s:default_choice_cmd = 'rg --files'
    elseif has('win32')
        let s:default_choice_cmd = 'dir /a-d /s /b | findstr /V "\\.git\\\\"'
    else
        let s:default_choice_cmd = "find . -type f -not -path '*/.git/*' -print"
    endif
    return s:default_choice_cmd
endfunction

function! heatseeker#command(choice_command, hs_args, first_command, rest_command) abort
    let l:choice_cmd = (a:choice_command ==# '' ? s:DefaultChoiceCommand() : a:choice_command)
    let l:hs = 'hs --full-screen'
    if a:hs_args !=# ''
        let l:hs .= ' ' . a:hs_args
    endif
    let l:pipeline = l:choice_cmd . ' | ' . l:hs
    let l:empty_tab = s:IsCurrentTabEmpty()

    " Prefer running inside a terminal/pty
    if has('nvim')
        " Neovim terminal path
        let l:tmp = tempname()
        let l:cmd = l:pipeline . ' > ' . shellescape(l:tmp)
        let l:return_win = win_getid()
        tabnew
        let l:term_win = win_getid()
        let l:bufnr = bufnr('%')
        let l:cb = {'tempfile': l:tmp, 'first': a:first_command, 'rest': a:rest_command, 'term_win': l:term_win, 'bufnr': l:bufnr, 'return_win': l:return_win, 'empty_tab': l:empty_tab}
        let l:OnExit = function('s:HsNvimOnExit', [l:cb])
        let l:job = jobstart(l:cmd, {'term': v:true, 'on_exit': l:OnExit})
        let l:auto_insert = get(g:, 'heatseeker_nvim_auto_insert', 1)
        if l:auto_insert
            startinsert
        endif
        return
    else
        " Synchronous fallback (no terminal support)
        try
            let l:sels = systemlist(l:pipeline)
            let l:sels = map(l:sels, 'substitute(v:val, "\r$", "", "")')
        catch /^Vim:Interrupt$/
            redraw!
            return
        endtry
        redraw!
        call s:HsContinuation(l:sels, a:first_command, a:rest_command, l:empty_tab)
    endif
endfunction

function! heatseeker#identifier() abort
    " Use current word without clobbering user registers
    let l:word = expand('<cword>')
    " Seed search with that word; quote for shell
    call heatseeker#command('', '-s ' . shellescape(l:word), ':edit', ':tabedit')
endfunction

function! heatseeker#buffer() abort
    let bufnrs = filter(range(1, bufnr("$")), 'buflisted(v:val)')
    let buffers = map(bufnrs, 'bufname(v:val)')
    let named_buffers = filter(buffers, '!empty(v:val)')
    if has('win32')
        let filename = tempname()
        call writefile(named_buffers, filename)
        call heatseeker#command("type " . filename, "", ":b", ":b")
        silent let _ = system("del " . filename)
    else
        call heatseeker#command('echo "' . join(named_buffers, "\n") . '"', "", ":b", ":b")
    endif
endfunction

" vim:set ft=vim et ts=4 sw=4 sts=4:
