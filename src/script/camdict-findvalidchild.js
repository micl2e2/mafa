
// RELEASE
console.log = function(){}
var send_back = arguments[arguments.length - 1];
var upath = arguments[0];


// DEBUG
// function send_back(strlist) { console.log("sendback:",strlist); } 
// var upath = [11,1,1,3,3];


clearInterval(window['camd-res']); // kill existing timer
window['camd-res'] = setInterval(try_send_back, 500)


function try_send_back() {

    var interested = document.body;
    for (let j = 0; j < upath.length; j++) {
	if (interested == undefined) {
	    console.log("undef", j, interested);
	    return; // give up
	} else {
	    console.log('following...');
	}
	interested = interested.childNodes[upath[j]];
    }

    if (interested != undefined) {
	var n_child = interested.childNodes.length;

	if (n_child == 0) {
	    return; // give up
	}

	// get all items' parent
	for (let i=0; i<n_child; i++) {
	    let cur = interested.childNodes[i];
	    if (cur.innerText != undefined &&
		cur.innerText.includes(
		    String.fromCharCode(0xa)
			+ "Add to word list "
			+ String.fromCharCode(0xa)
		)) {
		// upath.push(i);
		interested = interested.childNodes[i];
		break;
	    }
	}
	
	console.log("upath & interested", upath, interested);

	let send_res = "" // things sent back

	// children one by one
	for (let i=0; i<n_child; i++) {
	    let child = interested.childNodes[i];
	    if (child != undefined && child.nodeType == 1) {
		send_res += "______" + child.innerText;
	    }
	}

	send_back(send_res);
	clearInterval(window['camd-res']); // kill existing timer
    }

}

// document.body.childNodes[11].childNodes[1].childNodes[1].childNodes[3].childNodes[3].childNodes[x]


