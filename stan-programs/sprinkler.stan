data {
   int<lower=0, upper=1> sprinkler_on;
   int<lower=0, upper=1> grass_wet;
}
parameters {
   int<lower=0, upper=1> cloudy;
   int<lower=0, upper=1> raining;
}
model {
    // It's cloudy half the time.
    cloudy ~ bernoulli(0.5);

    if (cloudy) {
        // If it's cloudy it's usually raining.
        raining ~ bernoulli(0.8);
    } else {
        // Sometimes it rains when it's not cloudy.
        raining ~ bernoulli(0.2);
    }

    if (cloudy) {
        // We don't usually turn the sprinkler on when it's cloudy
        sprinkler_on ~ bernoulli(0.1);
    } else {
        // We turn it on half of the time when it's fine.
        sprinkler_on ~ bernoulli(0.5);
    }

    if (sprinkler_on && raining) {
        // The grass is almost certainly wet if it's raining and the sprinkler is on.
        grass_wet ~ bernoulli(0.99);
    } else if (sprinkler_on || raining) {
        // the sprinkler and the rain each have a 90% chance of making the grass wet
        grass_wet ~ bernoulli(0.9);
    } else {
        // if neither the sprinkler nor the rain are going then the grass is almost certainly dry.
        grass_wet ~ bernoulli(0.01);
    }
}
