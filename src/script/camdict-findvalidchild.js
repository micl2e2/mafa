var upath = [11,1,1,3,3];

var interested = document.body;
for (let j = 0; j < upath.length; j++) {
    if (interested == undefined) {
	console.log("undef", j, interested);
    } else {
	console.log('following...');
    }
    interested = interested.childNodes[upath[j]];
}

if (interested != undefined) {
    var n_child = interested.childNodes.length;

    for (let i = 0; i < n_child; i++) {
	let cur = interested.childNodes[i];
	if (cur.innerText != undefined &&
	    cur.innerText.includes("\nAdd to word list \n")) {
	    upath.push(i);
	}
    }
}

console.log("actual upath", upath)
// document.body.childNodes[11].childNodes[1].childNodes[1].childNodes[3].childNodes[3].childNodes[x]


