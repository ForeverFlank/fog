if exists('b:current_syntax') | finish|  endif

syntax match fogVar "\k\+" nextgroup=fogTypeAnnotate fogAssignment
syntax match fogTypeAnnotate ":" contained nextgroup=fogValue
syntax match fogAssignment "=" contained nextgroup=fogValue
syntax match fogValue ".*" contained

hi def link fogVar Identifier
hi def link fogAssignment Statement
hi def link fogValue String

let b:current_syntax = 'fog'