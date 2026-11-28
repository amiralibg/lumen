// Read or drive a running app's macOS accessibility tree, the way VoiceOver
// sees it. Build: swiftc -O scripts/ax.swift -o target/ax
//
//   ax <pid> dump [-v]            role, name, value, state (-v: frames, actions)
//   ax <pid> press <name>         AXPress
//   ax <pid> act <name> <AXAction>
//   ax <pid> set <name> <value>   set AXValue
//   ax <pid> focus <name>         set AXFocused
//   ax <pid> focused              the element that has keyboard focus
//
// The terminal running it needs Accessibility permission.
import ApplicationServices
import Foundation

let args = CommandLine.arguments
guard args.count >= 3, let pid = Int32(args[1]) else { print("usage: ax <pid> dump|press|act|set|focus ..."); exit(2) }
if !AXIsProcessTrusted() { print("not trusted: grant Accessibility to this terminal"); exit(3) }
let app = AXUIElementCreateApplication(pid)
AXUIElementSetMessagingTimeout(app, 5)

func attr(_ e: AXUIElement, _ a: String) -> AnyObject? {
    var v: AnyObject?
    return AXUIElementCopyAttributeValue(e, a as CFString, &v) == .success ? v : nil
}
func str(_ e: AXUIElement, _ a: String) -> String? {
    guard let v = attr(e, a) else { return nil }
    if let s = v as? String { return s.isEmpty ? nil : s }
    if let n = v as? NSNumber { return n.stringValue }
    return nil
}
func name(_ e: AXUIElement) -> String { str(e, "AXTitle") ?? str(e, "AXDescription") ?? "" }
func children(_ e: AXUIElement) -> [AXUIElement] { (attr(e, "AXChildren") as? [AXUIElement]) ?? [] }
func actions(_ e: AXUIElement) -> [String] {
    var n: CFArray?
    return AXUIElementCopyActionNames(e, &n) == .success ? (n as? [String] ?? []) : []
}
func frame(_ e: AXUIElement) -> String {
    var p = CGPoint.zero, s = CGSize.zero
    if let pv = attr(e, "AXPosition") { AXValueGetValue(pv as! AXValue, .cgPoint, &p) }
    if let sv = attr(e, "AXSize") { AXValueGetValue(sv as! AXValue, .cgSize, &s) }
    return String(format: "@%.0f,%.0f %.0fx%.0f", p.x, p.y, s.width, s.height)
}
func dump(_ e: AXUIElement, _ d: Int) {
    var line = String(repeating: "  ", count: d) + (str(e, "AXRole") ?? "?")
    if let s = str(e, "AXSubrole") { line += "/" + s }
    if let r = str(e, "AXRoleDescription") { line += " (\(r))" }
    let n = name(e); if !n.isEmpty { line += " \"\(n)\"" }
    if let v = str(e, "AXValue") { line += " = \(v)" }
    if let h = str(e, "AXHelp") { line += " help=\"\(h)\"" }
    if let p = str(e, "AXPlaceholderValue") { line += " placeholder=\"\(p)\"" }
    if str(e, "AXEnabled") == "0" { line += " [disabled]" }
    if str(e, "AXFocused") == "1" { line += " [focused]" }
    if str(e, "AXSelected") == "1" { line += " [selected]" }
    if let x = str(e, "AXExpanded") { line += " expanded=\(x)" }
    if args.contains("-v") { line += " " + frame(e) + " " + actions(e).joined(separator: ",") }
    print(line)
    for c in children(e) { dump(c, d + 1) }
}
func find(_ e: AXUIElement, _ title: String) -> AXUIElement? {
    if name(e) == title { return e }
    for c in children(e) { if let f = find(c, title) { return f } }
    return nil
}
var wins = (attr(app, "AXWindows") as? [AXUIElement]) ?? []
if wins.isEmpty, let f = attr(app, "AXFocusedWindow") { wins = [f as! AXUIElement] }
if wins.isEmpty { wins = children(app) }
guard let root = wins.first else { print("no windows"); exit(4) }
switch args[2] {
case "dump": dump(root, 0)
case "focused":
    if let f = attr(app, "AXFocusedUIElement") { dump(f as! AXUIElement, 0) } else { print("none") }
case "press", "act", "set", "focus":
    guard args.count >= 4, let el = find(root, args[3]) else { print("not found: \(args.count >= 4 ? args[3] : "")"); exit(5) }
    let r: AXError
    switch args[2] {
    case "press": r = AXUIElementPerformAction(el, "AXPress" as CFString)
    case "act": r = AXUIElementPerformAction(el, args[4] as CFString)
    case "set": r = AXUIElementSetAttributeValue(el, "AXValue" as CFString, args[4] as CFString)
    default: r = AXUIElementSetAttributeValue(el, "AXFocused" as CFString, kCFBooleanTrue)
    }
    print("\(args[2]) \(args[3]): \(r.rawValue)")
default: print("unknown command")
}
