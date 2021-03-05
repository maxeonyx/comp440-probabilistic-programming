
// data generated with python/numpy
// slope = 3
// intercept = 6
// x = np.linspace(0, 10, 11) + np.random.normal(0, 0.3, 11)
// y = ( x * slope + intercept ) + np.random.normal(0, 4, 11)

var data_x = [-0.38,  1.35,  1.78,  3.38,  4.08, 5.27,  5.91,  6.52,  7.89,  8.99, 10.06];
var data_y = [7.69, 13.30, 10.33, 16.06, 16.36, 17.65, 21.52, 23.77, 26.12, 41.85, 32.96];

var likelihood = function (slope, intercept, x) {
  var mu = x*slope + intercept;
  var sigma = 1.0;
  return Gaussian({ mu: mu, sigma: sigma });
};

var linear_regression = function () {
 
    var slope = gaussian({ mu: 0.0, sigma: 10.0 });
    var intercept = gaussian({ mu: 0.0, sigma: 10.0 });
    
    var do_observe = function (x, y) {
        observe(likelihood(slope, intercept, x), y);
    };
    map2(do_observe, data_x, data_y);
    
    return [slope, intercept];
};

viz.auto(Infer({method: 'MCMC', samples: 10000}, linear_regression));
