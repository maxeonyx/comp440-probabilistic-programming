var sides = 6;
var max_num_dice = 5;

var diceroll = function () {
  var num_dice = sample(RandomInteger({n: max_num_dice}));
  var a = num_dice >= 0 ? sample(RandomInteger({n: sides})) : 0;
  var b = num_dice >= 1 ? sample(RandomInteger({n: sides})) : 0;
  var c = num_dice >= 2 ? sample(RandomInteger({n: sides})) : 0;
  var d = num_dice >= 3 ? sample(RandomInteger({n: sides})) : 0;
  var e = num_dice >= 4 ? sample(RandomInteger({n: sides})) : 0;
  var sum = a + b + c + d + e;
  
  condition(sum === 5);
  
  return num_dice;
}

viz.auto(Infer(diceroll));

