// how many genes are there for height given these real-valued observations?
var observations = [1.6, 1.61];//, 1.65, 1.72, 1.73, 1.74, 1.82, 1.9];
var epsilon = 0.1;

var min_num_genes = 2
var max_num_genes = 8;

var height = function (num_genes) {
  var contributions = repeat(num_genes, function () {
    return sample(Uniform({a: 0.1, b: 0.9}));
  });
  
  return sum(contributions);
};

var heights = function () {
  var num_genes = sample(RandomInteger({n: max_num_genes - min_num_genes})) + min_num_genes;
  
  var height_0arg = function () {
    return height(num_genes);
  };
  
  var sample = sort(repeat(observations.length, height_0arg));
  
  var within_epsilon = function (a, b) {
    return (a >= b - epsilon) && (a <= b + epsilon);
  };
  
  var matches = map2(within_epsilon, sample, observations);
  
  var allmatch = all(function (a) {return a;}, matches);
  
  condition(allmatch);
  
  return num_genes;
}

viz.auto(Infer({ method: "rejection", samples: 10000 }, heights));

