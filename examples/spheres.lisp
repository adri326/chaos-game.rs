(define SHAPE (colorize
    (polygon 6)
    (list
        (srgb 200 16 250)
        (srgb 140 130 255)
        (srgb 160 220 255)
    )
))
(define SCALE 2.0)

(or-rule
    0.2
    (darken-rule (advance-rule (avoid2-choice 0 0) 1.5) 0.5)
    (or-rule
        0.5
        (advance-rule (avoid-choice 0) (/ 2.0 3.0) 0.75)
        (advance-rule (neighborhood-choice 1) (/ 2.0 3.0) 0.75)
    )
)
