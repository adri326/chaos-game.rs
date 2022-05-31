(define SCALE 2.0)
(define SHAPE (colorize
    (rotate (polygon 5) (/ PI 2.0))
    (interpolate (srgb 39 181 159) (darken (srgb 59 162 194) 0.5) 3)
))
(or-rule
    0.1
    (darken-rule
        (rand-advance-rule
            (neighborhood-choice 1)
            PHI
            0.1
            2.0
        )
        0.5
    )
    (advance-rule
        (avoid2-choice 0 0)
        (- PHI 1.0)
        0.66
    )
    0.4
)
