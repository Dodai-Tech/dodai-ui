// The id of the first window owned by the process named in $WINDOW_OWNER (default `frames`).
ObjC.import('CoreGraphics');ObjC.import('Foundation');
const owner=$.NSProcessInfo.processInfo.environment.objectForKey('WINDOW_OWNER');
const name=owner?ObjC.unwrap(owner):'frames';
const l=ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(1,0)));
const w=l.find(x=>x.kCGWindowOwnerName===name);
w?String(w.kCGWindowNumber):'';
