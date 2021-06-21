data {
    int<lower=0> N;
    vector[N] x;
    vector[N] y;
}
parameters {
    real slope;
    real bias;
}
model {
    slope ~ normal(0, 5);
    bias ~ normal(0, 5);
    y ~ normal(bias + slope * x, 1);
}
