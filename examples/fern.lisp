;; This is an implementation of the Barnsley Fern

(define SCALE 4.0)
(define CENTER (list 0.0 -5.0))

(define SHAPE (colorize (list
        '(0 0)
        '(0 -1.6)
        '(0 -1.6)
        '(0 -0.44)
    )
    (interpolate (srgb 64 200 128) (srgb 32 140 48) 4)
))

(define f1
    (affine-advance-rule
        (subset-choice (list 1 0 0 0))
        (list
            0 0
            0 0.16

            1 0
            0 1
        )
    )
)

(define f2
    (affine-advance-rule
        (subset-choice (list 0 1 0 0))
        (list
            0.85 -0.04
            0.04 0.85

            1 0
            0 1
        )
    )
)

(define f3
    (affine-advance-rule
        (subset-choice (list 0 0 1 0))
        (list
            0.2 0.26
            -0.23 0.22

            1 0
            0 1
        )
    )
)

(define f4
    (affine-advance-rule
        (subset-choice (list 0 0 0 1))
        (list
            -0.15 -0.28
            -0.26 0.24

            1 0
            0 1
        )
    )
)

(or-rules (list
    (list f1 0.01)
    (list f2 0.85)
    (list f3 0.07)
    (list f4 0.07)
))
