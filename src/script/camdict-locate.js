if (upaths.length > 0) {
    for (let i = 0; i < upaths.length; i++) {
	let cur_upath = upaths[i];
	var interested = document.body;
	for (let j = 0; j < cur_upath.length; j++) {
	    if (interested == undefined) {
		console.log("undef", j, interested);
	    } else {
		console.log('following...');
	    }
	    interested = interested.childNodes[cur_upath[j]];
	}
	console.log(i, interested);
    }
}


// hello
// [11,1,1,3,3,19,0,1,0,4,0,0,1,2,1,2,0,3,1]
// [11,1,1,3,3,19,1,1,0,4,0,0,1,2,1,2,0,3]
// document.body.childNodes[11].childNodes[1].childNodes[1].childNodes[3].childNodes[3].childNodes[19]
//
// prey
// [11,1,1,3,3,27,0,1,0,4,0,0,1,2,1,2,0,3,1]
// [11,1,1,3,3,27,1,1,0,4,0,0,1,2,1,2,0,3]
//
// telegram
// [11,1,1,3,3,17,0,1,0,4,0,0,1,2,1,2,0,3]
// [11,1,1,3,3,17,1,1,0,4,0,0,1,2,1,2,0,3]
