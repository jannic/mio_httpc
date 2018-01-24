extern crate mio_httpc;
extern crate mio;

use mio_httpc::{Request,CallBuilder,Httpc,WSPacket};
use mio::{Poll,Events};
// ws://demos.kaazing.com/echo

fn main() {
    let poll = Poll::new().unwrap();
    let mut htp = Httpc::new(10);
    let mut req = Request::builder();
    let args: Vec<String> = ::std::env::args().collect();
    let req = req.uri(args[1].as_str()).body(Vec::new()).expect("can not build request");

    let mut ws = CallBuilder::new(req).websocket(&mut htp, &poll).expect("Call start failed");

    let to = ::std::time::Duration::from_millis(800);
    'outer: loop {
        let mut events = Events::with_capacity(8);
        poll.poll(&mut events, Some(to)).unwrap();
        for cref in htp.timeout().into_iter() {
            if ws.is_ref(cref) {
                println!("Request timed out");
                break 'outer;
            }
        }

        if events.len() == 0 {
            // ws.ping(None);
            println!("send yo");
            ws.send_text(true, "yo!");
        }

        for ev in events.iter() {
            let cref = htp.event(&ev);

            if ws.is_call(&cref) {
                if ws.is_active() {
                    loop {
                        match ws.recv_packet(&mut htp, &poll).expect("Failed recv") {
                            WSPacket::Pong(_) => {
                                println!("Got pong!");
                            }
                            WSPacket::Ping(_) => {
                                println!("Got ping!");
                                ws.pong(None);
                            }
                            WSPacket::None => {
                                break;
                            }
                            WSPacket::Close(_,_) => {
                                println!("Got close!");
                                ws.close(None, None);
                                break 'outer;
                            }
                            WSPacket::Text(fin,txt) => {
                                println!("Got text={}, fin={}",txt,fin);
                            }
                            WSPacket::Binary(fin,b) => {
                                println!("Got bin={}B, fin={}",b.len(),fin);
                            }
                        }
                    }
                } else {
                    if ws.sendq_len() == 0 {
                        ws.ping(None);
                    }
                }
            }
        }
        // Any ping/pong/close/send_text/send_bin has just been buffered.
        // perform and recv_packet actually send over socket.
        ws.perform(&mut htp, &poll).expect("Call failed");
    }
    ws.perform(&mut htp, &poll).expect("can not perform");
    ws.finish(&mut htp);
}