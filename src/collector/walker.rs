use crate::collector::post_collector::PostCollector;
use crate::node::node::Node;
use crate::walk_utils::WalkerUtils;
use oxc_ast::ast::{
  BindingPatternKind, CallExpression, Expression, ImportSpecifier, ObjectPropertyKind, PropertyKey,
};
use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;
use oxc_syntax::node::NodeId;
use std::collections::HashMap;
use std::rc::Rc;
use log::debug;

pub struct Walker<'a> {
  pub node: Rc<Node>,
  pub semantic: &'a Semantic<'a>,
  pub i18n_namespaces: HashMap<String, Vec<String>>,
  pub post_collects: PostCollector,
  pub walk_utils: WalkerUtils<'a>,
}

impl<'a> Walker<'a> {
  pub fn new(node: Rc<Node>, semantic: &'a Semantic<'a>) -> Self {
    Self {
      node: node.clone(),
      semantic,
      i18n_namespaces: HashMap::new(),
      post_collects: PostCollector::new(),
      walk_utils: WalkerUtils::new(semantic, node.clone()),
    }
  }

  pub fn read_t(&mut self, symbol_id: SymbolId, namespace: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
          match node.kind() {
            AstKind::CallExpression(call) => {
              self.read_t_arguments(call, namespace.clone());
            }
            // TODO: handle bypass?
            _ => {}
          }
        }
      })
  }

  pub fn read_object_member_t(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {

        match node.kind() {
          // member call case
          // abc.t("abc")
          AstKind::MemberExpression(member) => {
            member.static_property_name().map(|prop| {
              if prop.eq("t") {
                if let Some(call_node) = self.semantic.nodes().parent_node(node.id()) {
                  if let AstKind::CallExpression(call) = call_node.kind() {
                    self.read_t_arguments(call, defined_ns.clone());
                  }
                }
              }
            });
          }
          // deconstruct case
          // const { t } = xyz;
          AstKind::VariableDeclaration(_var) => {
            // TODO: Deconstruct t from useTranslation case
          }
          _ => {}
        }
        }
      })
  }

  pub fn read_hook(&mut self, s: &oxc_allocator::Box<ImportSpecifier>, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(s.local.symbol_id.get().unwrap())
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
          // only call useTranslation method but not assignment
          if let Some(assign_node) = self.semantic.nodes().parent_node(node.id()) {

        // const xyz = useTranslation();
        let AstKind::VariableDeclarator(var) = assign_node.kind() else {
          return;
        };

        let namespace = match node.kind() {
          AstKind::CallExpression(call) => self.walk_utils.read_hook_namespace_argument(&call),
          _ => None,
        }
        .or_else(|| defined_ns.clone());

        match &var.id.kind {
          // const { t } = useTranslation();
          BindingPatternKind::ObjectPattern(obj) => obj.properties.iter().for_each(|prop| {
            if let PropertyKey::StaticIdentifier(key) = &prop.key {
              if key.name == "t" {
                if let BindingPatternKind::BindingIdentifier(ident) = &prop.value.kind {
                  self.read_t(ident.symbol_id(), namespace.clone());
                }
              }
            }
          }),
          // const trans = useTranslation();
          BindingPatternKind::BindingIdentifier(ident) => {
            self.read_object_member_t(ident.symbol_id(), namespace)
          }
          _ => {}
        }
          }
        }
      });
  }

  pub fn read_t_arguments(&mut self, call: &CallExpression, namespace: Option<String>) {
    let Some(arg) = call.arguments.get(0) else {
      return;
    };
    let key = arg
      .as_expression()
      .and_then(|expr| self.walk_utils.read_str_expression(&expr));

    let Some(key) = key else {
      // If we can't resolve the key, check if this is a dynamic key pattern
      if let Some(expr) = arg.as_expression() {
        if let Some(dynamic_keys) = self.try_resolve_dynamic_keys(expr) {
          for dynamic_key in dynamic_keys {
            self.add_key(&"default".to_string(), dynamic_key);
          }
          return;
        }
      }
      return;
    };

    let ns = call
      .arguments
      .get(1)
      .and_then(|arg| arg.as_expression())
      .and_then(|expr| {
        // t("abc", { ns: "xyz" })
        if let Expression::ObjectExpression(obj) = expr {
          obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
              return None;
            };

            let PropertyKey::StaticIdentifier(prop_name) = &obj_prop.key else {
              return None;
            };

            if prop_name.name == "ns" {
              return self.walk_utils.read_str_expression(&obj_prop.value);
            }
            None
          })
        } else {
          None
        }
      })
      .or_else(|| namespace.and_then(|ns| Some(ns)))
      .unwrap_or_else(|| "default".to_string());

    // Add the key directly without any hardcoded pattern matching
    self.add_key(&ns, key);
  }


  pub fn read_namespace_import(&mut self, symbol_id: SymbolId, members: &std::collections::HashMap<String, Option<crate::node::i18n_types::I18nMember>>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
        match node.kind() {
          // Handle member expressions like i18n.useTranslation() or i18n.t()
          oxc_ast::AstKind::MemberExpression(member) => {
            if let Some(prop_name) = member.static_property_name() {
              if let Some(Some(member_info)) = members.get(prop_name) {
                match member_info.r#type {
                  crate::node::i18n_types::I18nType::Hook => {
                    // Handle i18n.useTranslation()
                    if let Some(call_node) = self.semantic.nodes().parent_node(node.id()) {
                      if let oxc_ast::AstKind::CallExpression(call) = call_node.kind() {
                        self.read_hook_from_namespace(call, member_info.ns.clone());
                      }
                    }
                  }
                  crate::node::i18n_types::I18nType::TMethod => {
                    // Handle i18n.t()
                    if let Some(call_node) = self.semantic.nodes().parent_node(node.id()) {
                      if let oxc_ast::AstKind::CallExpression(call) = call_node.kind() {
                        self.read_t_arguments(call, member_info.ns.clone());
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
      });
  }

  pub fn read_hook_from_namespace(&mut self, call: &oxc_ast::ast::CallExpression, defined_ns: Option<String>) {
    // Similar to read_hook but for namespace calls
    let namespace = self.walk_utils.read_hook_namespace_argument(call)
      .or_else(|| defined_ns.clone())
      .unwrap_or_else(|| "default".to_string());

    // For namespace imports, we need to find the destructured variables
    // This is a simplified implementation - in practice you'd need to track
    // the destructuring pattern more carefully
    self.add_key(&namespace, "NAMESPACE_IMPORT".to_string());
  }

  pub fn read_custom_hook(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    // Handle custom hooks that wrap i18n functions
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
        match node.kind() {
          oxc_ast::AstKind::Function(func) => {
            if let Some(body) = &func.body {
              for stmt in &body.statements {
                self.read_statement_for_t_calls(stmt, defined_ns.clone());
              }
            }
          }
          oxc_ast::AstKind::VariableDeclarator(var) => {
            if let Some(init) = &var.init {
              if let oxc_ast::ast::Expression::ArrowFunctionExpression(arrow) = init {
                let body = &arrow.body;
                for stmt in &body.statements {
                  self.read_statement_for_t_calls(stmt, defined_ns.clone());
                }
              }
            }
          }
          _ => {}
        }
        }
      });
  }

  pub fn try_resolve_dynamic_keys(&self, _expr: &oxc_ast::ast::Expression) -> Option<Vec<String>> {
    // Try to resolve dynamic key patterns like keyPrefix + '_' + v
    // where v is a parameter that could have multiple values
    
    // This is a complex feature that would require:
    // 1. Detecting array.map() patterns
    // 2. Resolving the array values
    // 3. Substituting parameter values in expressions
    // 4. Generating keys for each array element
    //
    // TODO: Implement proper dynamic key resolution without hardcoding
    // For now, this is not implemented as it requires significant
    // analysis infrastructure beyond basic i18n key detection
    
    None
  }

  pub fn detect_custom_i18n_hooks(&mut self) {
    // Search for functions that call useTranslation and treat them as i18n functions
    for node in self.semantic.nodes().iter() {
      match node.kind() {
        oxc_ast::AstKind::Function(func) => {
          if let Some(body) = &func.body {
            if self.function_uses_use_translation(body) {
              // This function uses useTranslation, so it's a custom i18n hook
              for stmt in &body.statements {
                self.read_statement_for_t_calls(stmt, None);
              }
            }
          }
        }
        oxc_ast::AstKind::VariableDeclarator(var) => {
          if let Some(init) = &var.init {
            if let oxc_ast::ast::Expression::ArrowFunctionExpression(arrow) = init {
              let body = &arrow.body;
              if self.function_uses_use_translation(body) {
                // This function uses useTranslation, so it's a custom i18n hook
                for stmt in &body.statements {
                  self.read_statement_for_t_calls(stmt, None);
                }
              }
            }
          }
        }
        _ => {}
      }
    }
  }

  fn function_uses_use_translation(&self, body: &oxc_ast::ast::FunctionBody) -> bool {
    for stmt in &body.statements {
      if self.statement_uses_use_translation(stmt) {
        return true;
      }
    }
    false
  }

  fn statement_uses_use_translation(&self, stmt: &oxc_ast::ast::Statement) -> bool {
    match stmt {
      oxc_ast::ast::Statement::VariableDeclaration(var_decl) => {
        for declarator in &var_decl.declarations {
          if let Some(init) = &declarator.init {
            if self.expression_uses_use_translation(init) {
              return true;
            }
          }
        }
      }
      oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
        if self.expression_uses_use_translation(&expr_stmt.expression) {
          return true;
        }
      }
      oxc_ast::ast::Statement::ReturnStatement(ret_stmt) => {
        if let Some(expr) = &ret_stmt.argument {
          if self.expression_uses_use_translation(expr) {
            return true;
          }
        }
      }
      _ => {}
    }
    false
  }

  fn expression_uses_use_translation(&self, expr: &oxc_ast::ast::Expression) -> bool {
    match expr {
      oxc_ast::ast::Expression::CallExpression(call) => {
        match &call.callee {
          oxc_ast::ast::Expression::Identifier(ident) => {
            return ident.name == "useTranslation";
          }
          _ => false,
        }
      }
      _ => false,
    }
  }

  pub fn read_trans_component(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
          // Check if this is a JSX element name
          if let Some(_jsx_element) = node.kind().as_jsx_element() {
            if let Some(parent_node) = self.semantic.nodes().parent_node(node.id()) {
              if let Some(jsx_element) = parent_node.kind().as_jsx_element() {
                self.read_trans_jsx_element(jsx_element, defined_ns.clone());
              } else {
                // Try to go up one more level
                if let Some(grandparent_node) = self.semantic.nodes().parent_node(parent_node.id()) {
                  if let Some(jsx_element) = grandparent_node.kind().as_jsx_element() {
                    self.read_trans_jsx_element(jsx_element, defined_ns.clone());
                  }
                }
              }
            }
          } else {
            match node.kind() {
            // Handle JSX elements like <Trans i18nKey="key" />
            oxc_ast::AstKind::JSXElement(jsx_element) => {
              self.read_trans_jsx_element(jsx_element, defined_ns.clone());
            }
            // Handle JSX opening elements like <Trans i18nKey="key">
            oxc_ast::AstKind::JSXOpeningElement(opening_element) => {
              self.read_trans_jsx_opening_element(opening_element, defined_ns.clone());
            }
            _ => {}
          }
        }
        }
      });
  }

  pub fn read_trans_jsx_element(&mut self, jsx_element: &oxc_ast::ast::JSXElement, defined_ns: Option<String>) {
    let opening_element = &jsx_element.opening_element;
    self.read_trans_jsx_opening_element(opening_element, defined_ns);
  }

  pub fn read_trans_jsx_opening_element(&mut self, opening_element: &oxc_ast::ast::JSXOpeningElement, defined_ns: Option<String>) {
    // Look for i18nKey prop
    for attribute in &opening_element.attributes {
      if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = attribute {
        let attr_name = &attr.name;
        if let oxc_ast::ast::JSXAttributeName::Identifier(ident) = attr_name {
          if ident.name == "i18nKey" {
            if let Some(attr_value) = &attr.value {
              if let oxc_ast::ast::JSXAttributeValue::StringLiteral(s) = attr_value {
                let namespace = defined_ns.as_ref().map(|s| s.clone()).unwrap_or_else(|| "default".to_string());
                self.add_key(&namespace, s.value.to_string());
              }
            }
          }
        }
      }
    }
  }

  pub fn read_translation_component(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
          // Check if this is a JSX element name
          if let Some(_jsx_element) = node.kind().as_jsx_element() {
            if let Some(parent_node) = self.semantic.nodes().parent_node(node.id()) {
              if let Some(jsx_element) = parent_node.kind().as_jsx_element() {
                self.read_translation_jsx_element(jsx_element, defined_ns.clone());
              } else {
                // Try to go up one more level
                if let Some(grandparent_node) = self.semantic.nodes().parent_node(parent_node.id()) {
                  if let Some(jsx_element) = grandparent_node.kind().as_jsx_element() {
                    self.read_translation_jsx_element(jsx_element, defined_ns.clone());
                  }
                }
              }
            }
          } else {
            match node.kind() {
            // Handle JSX elements like <Translation>{(t) => <p>{t('key')}</p>}</Translation>
            oxc_ast::AstKind::JSXElement(jsx_element) => {
              self.read_translation_jsx_element(jsx_element, defined_ns.clone());
            }
            // Handle JSX opening elements like <Translation>
            oxc_ast::AstKind::JSXOpeningElement(_opening_element) => {
              // Need to find the parent JSX element to get the children
              if let Some(parent_node) = self.semantic.nodes().parent_node(node.id()) {
                if let oxc_ast::AstKind::JSXElement(jsx_element) = parent_node.kind() {
                  self.read_translation_jsx_element(jsx_element, defined_ns.clone());
                }
              }
            }
            _ => {}
          }
        }
        }
      });
  }

  pub fn read_translation_jsx_element(&mut self, jsx_element: &oxc_ast::ast::JSXElement, defined_ns: Option<String>) {
    // Translation component has children that are functions
    // We need to find t() calls within the children
    for child in &jsx_element.children {
      if let oxc_ast::ast::JSXChild::Element(child_element) = child {
        self.read_translation_jsx_element(child_element, defined_ns.clone());
      } else if let oxc_ast::ast::JSXChild::ExpressionContainer(expr_container) = child {
        // Handle expressions like {(t) => <p>{t('key')}</p>} or {t('key')}
        // JSXExpression inherits from Expression, so we can match on it
        match &expr_container.expression {
          oxc_ast::ast::JSXExpression::EmptyExpression(_) => {}
          _ => {
            // Convert JSXExpression to Expression for processing
            if let Some(expr) = expr_container.expression.as_expression() {
              self.read_translation_expression(expr, defined_ns.clone());
            }
          }
        }
      }
    }
  }

  pub fn read_translation_jsx_fragment(&mut self, jsx_fragment: &oxc_ast::ast::JSXFragment, defined_ns: Option<String>) {
    // JSX fragments can contain expressions like <>{t('key')}</>
    for child in &jsx_fragment.children {
      if let oxc_ast::ast::JSXChild::Element(child_element) = child {
        self.read_translation_jsx_element(child_element, defined_ns.clone());
      } else if let oxc_ast::ast::JSXChild::ExpressionContainer(expr_container) = child {
        // Handle expressions like {t('key')}
        match &expr_container.expression {
          oxc_ast::ast::JSXExpression::EmptyExpression(_) => {}
          _ => {
            // Convert JSXExpression to Expression for processing
            if let Some(expr) = expr_container.expression.as_expression() {
              self.read_translation_expression(expr, defined_ns.clone());
            }
          }
        }
      }
    }
  }

  pub fn read_translation_expression(&mut self, expr: &oxc_ast::ast::Expression, defined_ns: Option<String>) {
    match expr {
      oxc_ast::ast::Expression::ArrowFunctionExpression(arrow) => {
        // Handle arrow functions like (t) => <p>{t('key')}</p>
        let body = &arrow.body;
        // FunctionBody is a struct with statements field
        for stmt in &body.statements {
          self.read_statement_for_t_calls(stmt, defined_ns.clone());
        }
      }
      // Handle JSX elements directly
      oxc_ast::ast::Expression::JSXElement(jsx_element) => {
        self.read_translation_jsx_element(jsx_element, defined_ns.clone());
      }
      // Handle JSX fragments like <>{t('key')}</>
      oxc_ast::ast::Expression::JSXFragment(jsx_fragment) => {
        self.read_translation_jsx_fragment(jsx_fragment, defined_ns.clone());
      }
      // Handle call expressions like t('key')
      oxc_ast::ast::Expression::CallExpression(call) => {
        self.read_t_arguments(call, defined_ns.clone());
      }
      _ => {}
    }
  }

  pub fn read_statement_for_t_calls(&mut self, stmt: &oxc_ast::ast::Statement, defined_ns: Option<String>) {
    match stmt {
      oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
        let expr = &expr_stmt.expression;
        self.read_translation_expression(expr, defined_ns.clone());
      }
      oxc_ast::ast::Statement::ReturnStatement(ret_stmt) => {
        if let Some(expr) = &ret_stmt.argument {
          self.read_translation_expression(expr, defined_ns.clone());
        }
      }
      _ => {}
    }
  }

  pub fn read_hoc_wrapper(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
        match node.kind() {
          // Handle call expressions like withTranslation()(Component)
          oxc_ast::AstKind::CallExpression(call) => {
            self.read_hoc_call_expression(call, defined_ns.clone(), node.id());
          }
          _ => {}
        }
        }
      });
  }

  pub fn read_hoc_call_expression(&mut self, call: &oxc_ast::ast::CallExpression, defined_ns: Option<String>, node_id: NodeId) {
    // For HOC wrappers like withTranslation()(Component), we need to find the wrapped component
    // and look for t() calls within it
    if let Some(arg) = call.arguments.get(0) {
      if let Some(expr) = arg.as_expression() {
        match expr {
          oxc_ast::ast::Expression::Identifier(ident) => {
            // Find the component definition and look for t() calls
            self.read_component_for_t_calls(ident.reference_id(), defined_ns.clone());
          }
          _ => {}
        }
      }
    } else {
      // No arguments, this might be the first call withTranslation()
      // We need to look for the parent call expression that calls the result
      if let Some(parent_node) = self.semantic.nodes().parent_node(node_id) {
        if let oxc_ast::AstKind::CallExpression(parent_call) = parent_node.kind() {
          if let Some(arg) = parent_call.arguments.get(0) {
            if let Some(expr) = arg.as_expression() {
              match expr {
                oxc_ast::ast::Expression::Identifier(ident) => {
                  // For HOC components, we need to find the component definition
                  // and look for t() calls within it
                  self.find_component_definition_and_read_t_calls(ident.name.as_str(), defined_ns.clone());
                }
                _ => {}
              }
            }
          }
        }
      }
    }
  }

  pub fn read_component_for_t_calls(&mut self, ref_id: oxc_syntax::reference::ReferenceId, defined_ns: Option<String>) {
    self.read_component_for_t_calls_with_depth(ref_id, defined_ns, 0);
  }

  fn read_component_for_t_calls_with_depth(&mut self, ref_id: oxc_syntax::reference::ReferenceId, defined_ns: Option<String>, depth: usize) {
    if depth > 10 {
      debug!("Max depth reached, stopping recursion");
      return;
    }
    
    debug!("Reading component for t calls with ref_id: {:?}, depth: {}", ref_id, depth);
    if let Some(symbol_id) = self.semantic.scoping().get_reference(ref_id).symbol_id() {
      debug!("Found symbol_id: {:?}", symbol_id);
      self
        .semantic
        .symbol_references(symbol_id)
        .for_each(|ref_item| {
          if let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) {
          debug!("Component reference node kind: {:?}", node.kind());
          match node.kind() {
            oxc_ast::AstKind::Function(func) => {
              debug!("Found function definition for component");
              if let Some(body) = &func.body {
                for stmt in &body.statements {
                  self.read_statement_for_t_calls(stmt, defined_ns.clone());
                }
              }
            }
            // Handle JSX elements in component definitions
            oxc_ast::AstKind::JSXElement(jsx_element) => {
              debug!("Found JSX element in component");
              self.read_translation_jsx_element(jsx_element, defined_ns.clone());
            }
            // Handle variable declarations like const HocComp = ({ t }) => { ... }
            oxc_ast::AstKind::VariableDeclarator(var) => {
              debug!("Found variable declarator for component");
              if let Some(init) = &var.init {
                if let oxc_ast::ast::Expression::ArrowFunctionExpression(arrow) = init {
                  let body = &arrow.body;
                  // FunctionBody is a struct with statements field
                  for stmt in &body.statements {
                    self.read_statement_for_t_calls(stmt, defined_ns.clone());
                  }
                }
              }
            }
            // Handle arguments - skip them as they don't contain the component definition
            oxc_ast::AstKind::Argument(_) => {
              debug!("Found argument, skipping as it doesn't contain component definition");
            }
            _ => {}
          }
          }
        });
    }
  }

  pub fn find_component_definition_and_read_t_calls(&mut self, component_name: &str, defined_ns: Option<String>) {
    // Search through all nodes in the semantic tree to find the component definition
    for node in self.semantic.nodes().iter() {
      match node.kind() {
        oxc_ast::AstKind::VariableDeclarator(var) => {
          if let Some(init) = &var.init {
            if let oxc_ast::ast::Expression::ArrowFunctionExpression(arrow) = init {
              // Check if this is the component we're looking for
              if let Some(ident) = var.id.get_binding_identifier() {
                if ident.name == component_name {
                  // Found the component definition, look for t() calls within it
                  let body = &arrow.body;
                  for stmt in &body.statements {
                    self.read_statement_for_t_calls(stmt, defined_ns.clone());
                  }
                  return;
                }
              }
            }
          }
        }
        _ => {}
      }
    }
  }

  pub fn add_key(&mut self, namespace: &str, key: String) {
    let keys = self
      .i18n_namespaces
      .entry(namespace.to_string())
      .or_insert_with(Vec::new);
    
    // Only add if not already present
    if !keys.contains(&key) {
      keys.push(key);
    }
  }
}
