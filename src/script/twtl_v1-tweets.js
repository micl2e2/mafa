
// ONLY-DEBUG
// window['ulpath'] = {"upper_idx":[2,0,0,1,3,0,0,0,0,0,2,0,0,2,1,0,0,0],"lower_idx":[0,0,0,0,0,1,1,1]}

// document.body.childNodes[2].childNodes[0].childNodes[0].childNodes[1].childNodes[3].childNodes[0].childNodes[0].childNodes[0].childNodes[0].childNodes[0].childNodes[2].childNodes[0].childNodes[0].childNodes[2].childNodes[1].childNodes[0].childNodes[0].childNodes[0]

// RELEASE
console.log = function(){}

function get_loaded_nth(upper_last_elem) {
    let nchild = upper_last_elem.childNodes.length;
    if (nchild == 0)
	return 0;
    let ret = 0;
    for (let i=0; i<nchild; i++) {
	let nth_child = upper_last_elem.childNodes[i];
	if (nth_child == undefined) {
	    console.log("nth child null", nchild, upper_last_elem.childNodes);
	    return 0; // unlikely
	}
	if (nth_child.innerText != null &&
	    nth_child.innerText != undefined)
	    ret += 1;
	else
	    return ret;
    }
    return ret;
}

// ONLY-DEBUG
// function send_back(strlist) { console.log("here is~~~",strlist); } 
var send_back = arguments[arguments.length - 1];

clearInterval(window['get_tweets']); // kill existing timer
window['get_tweets'] = setInterval(function() {
    var ulpath = window['ulpath'];
    var parent_of_fork_nodes = document.body;
    // follow upper path
    for (let i = 0; i < ulpath.upper_idx.length; i++) {
	let u_idx = ulpath.upper_idx[i];
	if (parent_of_fork_nodes.childNodes.length > u_idx) {
	    parent_of_fork_nodes = parent_of_fork_nodes.childNodes[u_idx];
	}
    }
    
    window.ulpath.parent_of_fork_nodes = parent_of_fork_nodes;

    let loaded_n = get_loaded_nth(parent_of_fork_nodes);
    console.log("loaded_n", loaded_n);
    // if (is_nth_child_content_loaded(parent_of_fork_nodes)) {
    if (loaded_n > 0) {
	// var nchild = parent_of_fork_nodes.childNodes.length;
	var nchild = loaded_n;
	let tweets = [];
	console.log('nchild is',nchild);
	for (let i = 0; i < nchild; i++) {
	    // the node we are interested in, specifically its innerText
	    let node_interested = parent_of_fork_nodes.childNodes[i];
	    let tweet_id = node_interested.innerHTML.match('/status/([0-9]+)/');

	    // final
	    var tweet_ov = "twtl_v1" + String.fromCharCode(0xa);
	    
	    // append with tweet_id
	    if (tweet_id != null && tweet_id != undefined && tweet_id.length == 2)
		//  0 is whole match, 1 is what we want
		tweet_ov += tweet_id[1] +  String.fromCharCode(0xa);
	    else
		tweet_ov += "UNKNOWNID" + String.fromCharCode(0xa);
	    
	    // append with all others	  
	    tweet_ov += node_interested.innerText;
	    
	    tweets.push(tweet_ov);
	}
	console.log(tweets);
	send_back(tweets);
	clearInterval(window['get_tweets']); // cleanup, we dont need that anymore
    } else {  }
}, 1000)

