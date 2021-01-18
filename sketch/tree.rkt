#lang racket

(provide Inner
         Text
         parse-tree)

(define (parse-tree tree)
  (match tree
    [`(,tag ,bindings ,children ...)
     (Inner tag bindings (map parse-tree children))]
    [(? string?)
     (Text tree)]
    [else
     (error
      (format "Invalid form: ~a." tree))]))

(define (print-tree tree port mode)
  (match tree
    [(Inner t bs cs)
     (let ([tag (symbol->string t)]
           [attrs (format-attrs bs)])
       (if (self-closing? t)
           (write-string (format "<~a~a />" tag attrs) port)
           (begin
             (write-string (format "<~a~a>" tag attrs) port)
             (for-each (lambda (child)
                         (print-tree child
                                     port
                                     mode))
                       cs)
             (write-string (format "</~a>" tag) port))))]
    [(Text value)
     (write-string value port mode)]))

(define (self-closing? tag)
  (memv tag '(br meta)))

(define (coerce-to-string x)
  (match x
    [(? symbol?) (symbol->string x)]
    [else x]))

(define (format-attrs bindings)
  (match bindings
    ['() ""]
    [`((,name . ,value) ,bs ...)
     (format " ~a=\"~a\"~a"
             (symbol->string name)
             (coerce-to-string value)
             (format-attrs bs))]))

(struct Inner (tag-name bindings children)
  #:methods gen:custom-write
  [(define write-proc print-tree)])

(struct Text (value)
  #:methods gen:custom-write
  [(define write-proc print-tree)])
