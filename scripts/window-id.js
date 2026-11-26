ObjC.import('CoreGraphics');ObjC.import('Foundation');
const l=ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(1,0)));
const w=l.find(x=>x.kCGWindowOwnerName==='frames');
w?String(w.kCGWindowNumber):'';
