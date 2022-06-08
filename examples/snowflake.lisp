(define SHAPE (polygon 6))
(merge-rule
    (advance-rule (avoid-choice 0) (/ 2.0 3.0))
    (/ 3.0 4.0)
    (advance-rule (avoid-choice 0) (/ 2.0 3.0))
)
