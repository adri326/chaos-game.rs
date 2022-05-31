(define SCALE 2.0)
(define SHAPE (polygon 6))

(or-rule
    0.2
    (tensored-rule (discrete-spiral-rule
        (advance-rule (avoid-choice 0) (/ 4.0 3.0) 0.75)
        0.2
        (+ (/ PI 9.0) 0.01)
        0.75
        1.0
        0.5
    ))
    (tensor-rule
        (tensor-choice (choice) (avoid-choice 1))
        0.75
        (/ 2.0 3.0)
        0.75
        0.2
        F
    )
    0.6
)
