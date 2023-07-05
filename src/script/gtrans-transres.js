
// RELEASE
console.log = function() {}

// DEBUG
// function send_back(strlist) { console.log("here is~~~",strlist); } 
// RELEASE
var send_back = arguments[arguments.length - 1];

// DEBUG
// var upath = [4,0,1,0,1,0,1,1,2,1,1,9,0,3,0,0,1];
// RELEASE
var upath = arguments[0];

clearInterval(window['gtrans-res']); // kill existing timer
window['gtrans-res'] = setInterval(function() {
    var interested = document.body;
    if (upath.length > 0) {
	for (let i = 0; i < upath.length; i++) {
	    if (interested == undefined) {
		console.log(i);
		return;
	    } else {
		console.log(123);
	    }
	    interested = interested.childNodes[upath[i]];
	}
	console.log(interested);
	send_back(interested.innerText);
	clearInterval(window['gtrans-res']); // kill existing timer
    } else {
	console.log(upath);
    }
    
}, 500)

