" Location:     plugin/heatseeker.vim
" Author:       Ryan Schmitt
" Version:      1.0
" License:      MIT

if exists('g:loaded_heatseeker')
    finish
endif
let g:loaded_heatseeker = 1

" <Plug> mappings to avoid clobbering user choices; users can map to these.
nnoremap <silent> <Plug>(heatseeker-find) :call heatseeker#command('', '', ':edit', ':tabedit')<CR>
nnoremap <silent> <Plug>(heatseeker-identifier) :call heatseeker#identifier()<CR>
nnoremap <silent> <Plug>(heatseeker-buffer) :call heatseeker#buffer()<CR>

" Default keymaps (disable by setting g:heatseeker_default_mappings = 0)
if get(g:, 'heatseeker_default_mappings', 1)
    nnoremap <silent> <leader>f <Plug>(heatseeker-find)
    nnoremap <silent> <C-g> <Plug>(heatseeker-identifier)
    " Fuzzy select a buffer. Open the selected buffer with :b.
    nnoremap <silent> <leader>b <Plug>(heatseeker-buffer)
endif

" User commands
command! Heatseeker call heatseeker#command('', '', ':edit', ':tabedit')
command! HeatseekerIdentifier call heatseeker#identifier()
command! HeatseekerBuffer call heatseeker#buffer()

" vim:set ft=vim et ts=4 sw=4 sts=4:
