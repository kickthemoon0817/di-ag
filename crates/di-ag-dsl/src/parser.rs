use std::collections::HashMap;

use pest::Parser;
use pest_derive::Parser;

use di_ag_ir::*;

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct DiagParser;

struct ParseContext {
    variables: HashMap<String, VariableValue>,
}

enum VariableValue {
    String(String),
    Style(NodeStyle),
}

pub fn parse_dsl(input: &str) -> Result<Document, ParseError> {
    let pairs = DiagParser::parse(Rule::document, input)
        .map_err(|e| ParseError::Grammar(e.to_string()))?;

    let mut doc = Document::default();
    let mut ctx = ParseContext {
        variables: HashMap::new(),
    };
    let mut preset_type: Option<DiagramType> = None;
    let mut preset_theme: Option<String> = None;
    let mut preset_layout: Option<LayoutConfig> = None;

    for pair in pairs {
        if pair.as_rule() == Rule::document {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::node_stmt => {
                        let node = parse_node(inner, &ctx)?;
                        doc.nodes.push(node);
                    }
                    Rule::edge_stmt => {
                        let edge = parse_edge(inner, doc.edges.len())?;
                        doc.edges.push(edge);
                    }
                    Rule::container_stmt => {
                        let (container_node, edges) = parse_container(inner, &ctx)?;
                        doc.nodes.push(container_node);
                        doc.edges.extend(edges);
                    }
                    Rule::directive => {
                        parse_directive(
                            inner,
                            &mut preset_type,
                            &mut preset_theme,
                            &mut preset_layout,
                        )?;
                    }
                    Rule::let_stmt => {
                        parse_let(inner, &mut ctx)?;
                    }
                    Rule::align_stmt | Rule::distribute_stmt => {
                        // Layout directives stored for layout engine — no-op in parsing
                    }
                    _ => {}
                }
            }
        }
    }

    if preset_type.is_some() || preset_theme.is_some() || preset_layout.is_some() {
        doc.preset = Some(Preset {
            diagram_type: preset_type.unwrap_or_default(),
            theme: preset_theme,
            layout: preset_layout,
        });
    }

    Ok(doc)
}

fn parse_directive(
    pair: pest::iterators::Pair<Rule>,
    preset_type: &mut Option<DiagramType>,
    preset_theme: &mut Option<String>,
    preset_layout: &mut Option<LayoutConfig>,
) -> Result<(), ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::preset_directive => {
            let name = inner.into_inner().next().unwrap().as_str();
            *preset_type = Some(match name {
                "flowchart" => DiagramType::Flowchart,
                "sequence" => DiagramType::Sequence,
                "er" => DiagramType::Er,
                "class" => DiagramType::Class,
                "tree" => DiagramType::Tree,
                "freeform" => DiagramType::Freeform,
                other => return Err(ParseError::UnknownDiagramType(other.into())),
            });
        }
        Rule::theme_directive => {
            let name = inner.into_inner().next().unwrap().as_str();
            *preset_theme = Some(name.into());
        }
        Rule::layout_directive => {
            let mut direction = LayoutDirection::default();
            let mut spacing = 60.0;
            for kv in inner.into_inner() {
                if kv.as_rule() == Rule::key_value_pair {
                    let mut parts = kv.into_inner();
                    let key = parts.next().unwrap().as_str();
                    let val = parts.next().unwrap();
                    let val_str = extract_value_str(&val);
                    match key {
                        "direction" => {
                            direction = match val_str.as_str() {
                                "TB" => LayoutDirection::TopToBottom,
                                "BT" => LayoutDirection::BottomToTop,
                                "LR" => LayoutDirection::LeftToRight,
                                "RL" => LayoutDirection::RightToLeft,
                                other => {
                                    return Err(ParseError::UnknownDirection(other.into()))
                                }
                            };
                        }
                        "spacing" => {
                            spacing = val_str
                                .parse::<f64>()
                                .map_err(|_| ParseError::InvalidNumber(val_str))?;
                        }
                        _ => {}
                    }
                }
            }
            *preset_layout = Some(LayoutConfig { direction, spacing });
        }
        _ => {}
    }
    Ok(())
}

fn parse_node(
    pair: pest::iterators::Pair<Rule>,
    ctx: &ParseContext,
) -> Result<Node, ParseError> {
    let mut inner = pair.into_inner();
    let id = inner.next().unwrap().as_str().to_string();
    let mut label = id.clone();
    let mut shape = Shape::default();
    let mut size = None;
    let mut style = NodeStyle::default();
    let mut ports = vec![];

    for part in inner {
        match part.as_rule() {
            Rule::quoted_string => {
                label = unquote(part.as_str());
            }
            Rule::node_block => {
                for prop in part.into_inner() {
                    if prop.as_rule() == Rule::node_property {
                        let p = prop.into_inner().next().unwrap();
                        match p.as_rule() {
                            Rule::shape_prop => {
                                let name = p.into_inner().next().unwrap().as_str();
                                shape = parse_shape(name)?;
                            }
                            Rule::size_prop => {
                                let mut nums = p.into_inner();
                                let w: f64 = nums
                                    .next()
                                    .unwrap()
                                    .as_str()
                                    .parse()
                                    .map_err(|_| ParseError::InvalidNumber("width".into()))?;
                                let h: f64 = nums
                                    .next()
                                    .unwrap()
                                    .as_str()
                                    .parse()
                                    .map_err(|_| ParseError::InvalidNumber("height".into()))?;
                                size = Some(Size {
                                    width: w,
                                    height: h,
                                });
                            }
                            Rule::style_prop => {
                                style =
                                    parse_style_block(p.into_inner().next().unwrap(), ctx)?;
                            }
                            Rule::port_prop => {
                                let list = p.into_inner().next().unwrap();
                                for port_id in list.into_inner() {
                                    if port_id.as_rule() == Rule::identifier {
                                        ports.push(match port_id.as_str() {
                                            "top" => Port::Top,
                                            "bottom" => Port::Bottom,
                                            "left" => Port::Left,
                                            "right" => Port::Right,
                                            other => Port::Custom(other.into()),
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Node {
        id,
        label,
        shape,
        position: None,
        size,
        style,
        ports,
        children: vec![],
    })
}

fn parse_edge(
    pair: pest::iterators::Pair<Rule>,
    index: usize,
) -> Result<Edge, ParseError> {
    let mut inner = pair.into_inner();
    let source = inner.next().unwrap().as_str().to_string();
    let target = inner.next().unwrap().as_str().to_string();
    let mut label = None;
    let mut edge_type = EdgeType::default();
    let mut style = EdgeStyle::default();
    let mut waypoints = vec![];

    for part in inner {
        if part.as_rule() == Rule::edge_block {
            for prop in part.into_inner() {
                if prop.as_rule() == Rule::edge_property {
                    let p = prop.into_inner().next().unwrap();
                    match p.as_rule() {
                        Rule::label_prop => {
                            let qs = p.into_inner().next().unwrap();
                            label = Some(unquote(qs.as_str()));
                        }
                        Rule::route_prop => {
                            let name = p.into_inner().next().unwrap().as_str();
                            edge_type = match name {
                                "straight" => EdgeType::Straight,
                                "curved" => EdgeType::Curved,
                                "orthogonal" => EdgeType::Orthogonal,
                                other => return Err(ParseError::UnknownRoute(other.into())),
                            };
                        }
                        Rule::style_prop => {
                            let block = p.into_inner().next().unwrap();
                            // Parse style entries for edge
                            for entry in block.into_inner() {
                                if entry.as_rule() == Rule::style_entry {
                                    let mut parts = entry.into_inner();
                                    let key = parts.next().unwrap().as_str();
                                    let val = parts.next().unwrap();
                                    let val_str = extract_value_str(&val);
                                    match key {
                                        "stroke" => style.stroke = Some(val_str),
                                        "color" => style.color = Some(val_str),
                                        "stroke_width" => {
                                            style.stroke_width = val_str.parse().ok()
                                        }
                                        "dash" => style.dash = Some(val_str),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Rule::waypoints_prop => {
                            let list = p.into_inner().next().unwrap();
                            for wp in list.into_inner() {
                                if wp.as_rule() == Rule::waypoint {
                                    let mut nums = wp.into_inner();
                                    let x: f64 =
                                        nums.next().unwrap().as_str().parse().unwrap_or(0.0);
                                    let y: f64 =
                                        nums.next().unwrap().as_str().parse().unwrap_or(0.0);
                                    waypoints.push(Waypoint { x, y });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(Edge {
        id: format!("e{}", index),
        source,
        target,
        label,
        edge_type,
        waypoints,
        style,
    })
}

fn parse_container(
    pair: pest::iterators::Pair<Rule>,
    ctx: &ParseContext,
) -> Result<(Node, Vec<Edge>), ParseError> {
    let mut inner = pair.into_inner();
    let id = inner.next().unwrap().as_str().to_string();
    let mut label = id.clone();
    let mut children = vec![];
    let mut edges = vec![];
    let style = NodeStyle::default();

    for part in inner {
        match part.as_rule() {
            Rule::quoted_string => {
                label = unquote(part.as_str());
            }
            Rule::container_block => {
                let mut edge_count = 0;
                for body in part.into_inner() {
                    if body.as_rule() == Rule::container_body {
                        let stmt = body.into_inner().next().unwrap();
                        match stmt.as_rule() {
                            Rule::node_stmt => {
                                let node = parse_node(stmt, ctx)?;
                                children.push(node);
                            }
                            Rule::edge_stmt => {
                                let edge = parse_edge(stmt, edge_count)?;
                                edges.push(edge);
                                edge_count += 1;
                            }
                            Rule::container_stmt => {
                                let (child_container, child_edges) =
                                    parse_container(stmt, ctx)?;
                                children.push(child_container);
                                edges.extend(child_edges);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let node = Node {
        id,
        label,
        shape: Shape::Rect,
        position: None,
        size: None,
        style,
        ports: vec![],
        children,
    };

    Ok((node, edges))
}

fn parse_let(
    pair: pest::iterators::Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(), ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let val = inner.next().unwrap();
    match val.as_rule() {
        Rule::style_block => {
            let style = parse_style_block(val, ctx)?;
            ctx.variables.insert(name, VariableValue::Style(style));
        }
        Rule::value => {
            let s = extract_value_str(&val);
            ctx.variables.insert(name, VariableValue::String(s));
        }
        _ => {}
    }
    Ok(())
}

fn parse_style_block(
    pair: pest::iterators::Pair<Rule>,
    ctx: &ParseContext,
) -> Result<NodeStyle, ParseError> {
    let mut style = NodeStyle::default();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::style_entry {
            let mut parts = entry.into_inner();
            let key = parts.next().unwrap().as_str();
            let val = parts.next().unwrap();
            let val_str = resolve_value(&val, ctx)?;
            match key {
                "fill" => style.fill = Some(val_str),
                "stroke" => style.stroke = Some(val_str),
                "stroke_width" => style.stroke_width = val_str.parse().ok(),
                "font_family" => style.font_family = Some(val_str),
                "font_size" => style.font_size = val_str.parse().ok(),
                "font_color" | "color" => style.font_color = Some(val_str),
                "border_radius" | "radius" => style.border_radius = val_str.parse().ok(),
                "opacity" => style.opacity = val_str.parse().ok(),
                _ => {}
            }
        }
    }
    Ok(style)
}

fn parse_shape(name: &str) -> Result<Shape, ParseError> {
    match name {
        "rect" => Ok(Shape::Rect),
        "rounded_rect" | "rounded" => Ok(Shape::RoundedRect),
        "diamond" => Ok(Shape::Diamond),
        "circle" => Ok(Shape::Circle),
        "ellipse" => Ok(Shape::Ellipse),
        "cylinder" => Ok(Shape::Cylinder),
        "parallelogram" => Ok(Shape::Parallelogram),
        "hexagon" => Ok(Shape::Hexagon),
        "triangle" => Ok(Shape::Triangle),
        other => Err(ParseError::UnknownShape(other.into())),
    }
}

fn extract_value_str(pair: &pest::iterators::Pair<Rule>) -> String {
    let inner = pair.clone().into_inner().next();
    if let Some(inner) = inner {
        match inner.as_rule() {
            Rule::quoted_string | Rule::color_hex => unquote(inner.as_str()),
            Rule::variable_ref => inner.as_str()[1..].to_string(),
            _ => inner.as_str().to_string(),
        }
    } else {
        pair.as_str().to_string()
    }
}

fn resolve_value(
    pair: &pest::iterators::Pair<Rule>,
    ctx: &ParseContext,
) -> Result<String, ParseError> {
    let inner = pair.clone().into_inner().next();
    if let Some(inner) = inner {
        match inner.as_rule() {
            Rule::variable_ref => {
                let var_name = &inner.as_str()[1..];
                match ctx.variables.get(var_name) {
                    Some(VariableValue::String(s)) => Ok(s.clone()),
                    Some(VariableValue::Style(_)) => Ok(format!("<style:{}>", var_name)),
                    None => Err(ParseError::UndefinedVariable(var_name.into())),
                }
            }
            Rule::quoted_string | Rule::color_hex => Ok(unquote(inner.as_str())),
            _ => Ok(inner.as_str().to_string()),
        }
    } else {
        Ok(pair.as_str().to_string())
    }
}

fn unquote(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
