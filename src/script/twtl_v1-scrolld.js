// RELEASE
console.log = function() {}

var ulpath = window['ulpath'];
var parent_of_fork_nodes = ulpath.parent_of_fork_nodes;

var n_most_recent_loaded = arguments[0];

var nchild = parent_of_fork_nodes.childNodes.length;
let nth_child = parent_of_fork_nodes.childNodes[n_most_recent_loaded-1];
if (nth_child == undefined) {
    console.log("scroll fail",parent_of_fork_nodes.childNodes.length,n_most_recent_loaded);
} else {
    console.log("scroll good");
    nth_child.scrollIntoView();
}
