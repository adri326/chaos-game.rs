(define PHI (/ (+ (sqrt 5.0) 1.0) 2.0))

;; (define polygon (lambda (n) (map
;;     (lambda (i) (let ((x (/ (float i) (float n))))
;;         (list x)
;;     ))
;;     (range 0 n)
;; )))

;; (define SHAPE (polygon 7))

(or-rule
    0.4
    (advance-rule 0.5 0.5 (avoid2-choice 0 0))
    (advance-rule (/ 2.0 3.0) 0.5 (avoid-choice 0))
)
