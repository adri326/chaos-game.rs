(define CENTER (list -0.5 0.5))

(define SHAPE (list
    '(0 0)
    '(-1 1)
))

(define f1 (affine-advance-rule
    (subset-choice '(1 0))
    (list
        0.5 -0.5
        0.5 0.5

        0.5 0.0
        0.0 0.5
    )
))

(define f2 (affine-advance-rule
    (subset-choice '(0 1))
    (list
        0.5 0.5
        -0.5 0.5

        0.5 0.0
        0.0 0.5
    )
))

(or-rule 0.5 f1 f2)
