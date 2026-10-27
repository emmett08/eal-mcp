// Generated from grammar/EAL.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
#![allow(unused_parens)]
use antlr4rust::PredictionContextCache;
use antlr4rust::parser::{Parser, BaseParser, ParserRecog, ParserNodeType};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::TokenSource;
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::errors::*;
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::recognizer::{Recognizer,Actions};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::error_strategy::{ErrorStrategy, DefaultErrorStrategy};
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext,cast,cast_mut};
use antlr4rust::tree::*;
use antlr4rust::token::{TOKEN_EOF,OwningToken,Token};
use antlr4rust::int_stream::EOF;
use antlr4rust::vocabulary::{Vocabulary,VocabularyImpl};
use antlr4rust::token_factory::{CommonTokenFactory,TokenFactory, TokenAware};
use super::eallistener::*;
use super::ealvisitor::*;

use antlr4rust::lazy_static;
use antlr4rust::{TidAble,TidExt};

use std::marker::PhantomData;
use std::sync::Arc;
use std::rc::Rc;
use std::convert::TryFrom;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use std::borrow::{Borrow,BorrowMut};
use std::any::{Any,TypeId};

const _: () = {
    assert!(
        antlr4rust::const_check::version("0", "6"),
        "Incompatible version: either switch to v0.6 or regenerate with this version"
    );
};

		pub const EAL_T__0:i32=1; 
		pub const EAL_T__1:i32=2; 
		pub const EAL_T__2:i32=3; 
		pub const EAL_T__3:i32=4; 
		pub const EAL_T__4:i32=5; 
		pub const EAL_T__5:i32=6; 
		pub const EAL_T__6:i32=7; 
		pub const EAL_T__7:i32=8; 
		pub const EAL_T__8:i32=9; 
		pub const EAL_T__9:i32=10; 
		pub const EAL_T__10:i32=11; 
		pub const EAL_T__11:i32=12; 
		pub const EAL_T__12:i32=13; 
		pub const EAL_T__13:i32=14; 
		pub const EAL_T__14:i32=15; 
		pub const EAL_T__15:i32=16; 
		pub const EAL_T__16:i32=17; 
		pub const EAL_T__17:i32=18; 
		pub const EAL_T__18:i32=19; 
		pub const EAL_T__19:i32=20; 
		pub const EAL_T__20:i32=21; 
		pub const EAL_T__21:i32=22; 
		pub const EAL_T__22:i32=23; 
		pub const EAL_T__23:i32=24; 
		pub const EAL_T__24:i32=25; 
		pub const EAL_T__25:i32=26; 
		pub const EAL_T__26:i32=27; 
		pub const EAL_T__27:i32=28; 
		pub const EAL_T__28:i32=29; 
		pub const EAL_T__29:i32=30; 
		pub const EAL_T__30:i32=31; 
		pub const EAL_T__31:i32=32; 
		pub const EAL_T__32:i32=33; 
		pub const EAL_T__33:i32=34; 
		pub const EAL_T__34:i32=35; 
		pub const EAL_T__35:i32=36; 
		pub const EAL_T__36:i32=37; 
		pub const EAL_T__37:i32=38; 
		pub const EAL_T__38:i32=39; 
		pub const EAL_T__39:i32=40; 
		pub const EAL_T__40:i32=41; 
		pub const EAL_T__41:i32=42; 
		pub const EAL_T__42:i32=43; 
		pub const EAL_T__43:i32=44; 
		pub const EAL_T__44:i32=45; 
		pub const EAL_T__45:i32=46; 
		pub const EAL_T__46:i32=47; 
		pub const EAL_T__47:i32=48; 
		pub const EAL_T__48:i32=49; 
		pub const EAL_T__49:i32=50; 
		pub const EAL_T__50:i32=51; 
		pub const EAL_T__51:i32=52; 
		pub const EAL_T__52:i32=53; 
		pub const EAL_T__53:i32=54; 
		pub const EAL_T__54:i32=55; 
		pub const EAL_T__55:i32=56; 
		pub const EAL_T__56:i32=57; 
		pub const EAL_T__57:i32=58; 
		pub const EAL_T__58:i32=59; 
		pub const EAL_T__59:i32=60; 
		pub const EAL_T__60:i32=61; 
		pub const EAL_T__61:i32=62; 
		pub const EAL_T__62:i32=63; 
		pub const EAL_T__63:i32=64; 
		pub const EAL_T__64:i32=65; 
		pub const EAL_T__65:i32=66; 
		pub const EAL_T__66:i32=67; 
		pub const EAL_T__67:i32=68; 
		pub const EAL_T__68:i32=69; 
		pub const EAL_T__69:i32=70; 
		pub const EAL_T__70:i32=71; 
		pub const EAL_T__71:i32=72; 
		pub const EAL_T__72:i32=73; 
		pub const EAL_T__73:i32=74; 
		pub const EAL_T__74:i32=75; 
		pub const EAL_T__75:i32=76; 
		pub const EAL_T__76:i32=77; 
		pub const EAL_T__77:i32=78; 
		pub const EAL_T__78:i32=79; 
		pub const EAL_T__79:i32=80; 
		pub const EAL_T__80:i32=81; 
		pub const EAL_T__81:i32=82; 
		pub const EAL_T__82:i32=83; 
		pub const EAL_T__83:i32=84; 
		pub const EAL_T__84:i32=85; 
		pub const EAL_ID:i32=86; 
		pub const EAL_NUMBER:i32=87; 
		pub const EAL_STRING:i32=88; 
		pub const EAL_LINE_COMMENT:i32=89; 
		pub const EAL_BLOCK_COMMENT:i32=90; 
		pub const EAL_NL:i32=91; 
		pub const EAL_WS:i32=92;
	pub const EAL_EOF:i32=EOF;
	pub const RULE_program:usize = 0; 
	pub const RULE_declaration:usize = 1; 
	pub const RULE_moduleDecl:usize = 2; 
	pub const RULE_importDecl:usize = 3; 
	pub const RULE_contextDecl:usize = 4; 
	pub const RULE_contextAttribute:usize = 5; 
	pub const RULE_environmentDecl:usize = 6; 
	pub const RULE_toolDecl:usize = 7; 
	pub const RULE_versionField:usize = 8; 
	pub const RULE_evidenceDecl:usize = 9; 
	pub const RULE_evidenceField:usize = 10; 
	pub const RULE_toolField:usize = 11; 
	pub const RULE_kindField:usize = 12; 
	pub const RULE_environmentField:usize = 13; 
	pub const RULE_maxAgeField:usize = 14; 
	pub const RULE_inputField:usize = 15; 
	pub const RULE_inputBindingField:usize = 16; 
	pub const RULE_deriveField:usize = 17; 
	pub const RULE_assumptionDecl:usize = 18; 
	pub const RULE_assumptionField:usize = 19; 
	pub const RULE_statementField:usize = 20; 
	pub const RULE_validateField:usize = 21; 
	pub const RULE_validFromField:usize = 22; 
	pub const RULE_validUntilField:usize = 23; 
	pub const RULE_reasoningDecl:usize = 24; 
	pub const RULE_reasoningField:usize = 25; 
	pub const RULE_methodField:usize = 26; 
	pub const RULE_rationaleField:usize = 27; 
	pub const RULE_backingField:usize = 28; 
	pub const RULE_transferField:usize = 29; 
	pub const RULE_claimDecl:usize = 30; 
	pub const RULE_claimField:usize = 31; 
	pub const RULE_propositionDecl:usize = 32; 
	pub const RULE_propositionField:usize = 33; 
	pub const RULE_subjectField:usize = 34; 
	pub const RULE_quantityField:usize = 35; 
	pub const RULE_unitField:usize = 36; 
	pub const RULE_scopeField:usize = 37; 
	pub const RULE_queryField:usize = 38; 
	pub const RULE_resultField:usize = 39; 
	pub const RULE_argumentDecl:usize = 40; 
	pub const RULE_argumentFlow:usize = 41; 
	pub const RULE_support:usize = 42; 
	pub const RULE_supportGroup:usize = 43; 
	pub const RULE_supportKind:usize = 44; 
	pub const RULE_patternDecl:usize = 45; 
	pub const RULE_patternParameter:usize = 46; 
	pub const RULE_parameterKind:usize = 47; 
	pub const RULE_applicationDecl:usize = 48; 
	pub const RULE_patternBinding:usize = 49; 
	pub const RULE_bindingValue:usize = 50; 
	pub const RULE_patternGuard:usize = 51; 
	pub const RULE_objectionDecl:usize = 52; 
	pub const RULE_objectionSupport:usize = 53; 
	pub const RULE_objectionGroup:usize = 54; 
	pub const RULE_objectionSupportKind:usize = 55; 
	pub const RULE_targetKind:usize = 56; 
	pub const RULE_argumentationDirective:usize = 57; 
	pub const RULE_referenceList:usize = 58; 
	pub const RULE_reference:usize = 59; 
	pub const RULE_qualifiedName:usize = 60; 
	pub const RULE_predicate:usize = 61; 
	pub const RULE_expression:usize = 62; 
	pub const RULE_orExpression:usize = 63; 
	pub const RULE_andExpression:usize = 64; 
	pub const RULE_comparisonExpression:usize = 65; 
	pub const RULE_additiveExpression:usize = 66; 
	pub const RULE_multiplicativeExpression:usize = 67; 
	pub const RULE_unaryExpression:usize = 68; 
	pub const RULE_primaryExpression:usize = 69; 
	pub const RULE_comparator:usize = 70; 
	pub const RULE_jsonValue:usize = 71; 
	pub const RULE_jsonObject:usize = 72; 
	pub const RULE_jsonArray:usize = 73; 
	pub const RULE_jsonScalar:usize = 74; 
	pub const RULE_signedNumber:usize = 75; 
	pub const RULE_identifier:usize = 76; 
	pub const RULE_lineEnd:usize = 77;
	pub const ruleNames: [&'static str; 78] =  [
		"program", "declaration", "moduleDecl", "importDecl", "contextDecl", "contextAttribute", 
		"environmentDecl", "toolDecl", "versionField", "evidenceDecl", "evidenceField", 
		"toolField", "kindField", "environmentField", "maxAgeField", "inputField", 
		"inputBindingField", "deriveField", "assumptionDecl", "assumptionField", 
		"statementField", "validateField", "validFromField", "validUntilField", 
		"reasoningDecl", "reasoningField", "methodField", "rationaleField", "backingField", 
		"transferField", "claimDecl", "claimField", "propositionDecl", "propositionField", 
		"subjectField", "quantityField", "unitField", "scopeField", "queryField", 
		"resultField", "argumentDecl", "argumentFlow", "support", "supportGroup", 
		"supportKind", "patternDecl", "patternParameter", "parameterKind", "applicationDecl", 
		"patternBinding", "bindingValue", "patternGuard", "objectionDecl", "objectionSupport", 
		"objectionGroup", "objectionSupportKind", "targetKind", "argumentationDirective", 
		"referenceList", "reference", "qualifiedName", "predicate", "expression", 
		"orExpression", "andExpression", "comparisonExpression", "additiveExpression", 
		"multiplicativeExpression", "unaryExpression", "primaryExpression", "comparator", 
		"jsonValue", "jsonObject", "jsonArray", "jsonScalar", "signedNumber", 
		"identifier", "lineEnd"
	];


	pub const _LITERAL_NAMES: [Option<&'static str>;86] = [
		None, Some("'language'"), Some("'profile'"), Some("'module'"), Some("'{'"), 
		Some("'}'"), Some("'import'"), Some("'as'"), Some("'context'"), Some("','"), 
		Some("'environment'"), Some("'tool'"), Some("'max_age'"), Some("'kind'"), 
		Some("'input'"), Some("'version'"), Some("'validate'"), Some("'method'"), 
		Some("'subject'"), Some("'quantity'"), Some("'unit'"), Some("'scope'"), 
		Some("'valid_from'"), Some("'valid_until'"), Some("'query'"), Some("'evidence'"), 
		Some("'bind'"), Some("'from'"), Some("'derive'"), Some("'assumption'"), 
		Some("'statement'"), Some("'reasoning'"), Some("'rationale'"), Some("'backing'"), 
		Some("'transfer'"), Some("'to'"), Some("'assuming'"), Some("'reviewed'"), 
		Some("'claim'"), Some("'proposition'"), Some("'result'"), Some("'argument'"), 
		Some("'='"), Some("'via'"), Some("'=>'"), Some("'binding'"), Some("'['"), 
		Some("']'"), Some("'assumptions'"), Some("'premises'"), Some("'pattern'"), 
		Some("'('"), Some("')'"), Some("'decreases'"), Some("':'"), Some("'apply'"), 
		Some("'when'"), Some("'else'"), Some("'objection'"), Some("'-x>'"), Some("'strict'"), 
		Some("'rank'"), Some("'contrary'"), Some("'prefer'"), Some("'over'"), 
		Some("'head'"), Some("'tail'"), Some("'.'"), Some("'require'"), Some("'or'"), 
		Some("'and'"), Some("'+'"), Some("'-'"), Some("'*'"), Some("'/'"), Some("'%'"), 
		Some("'not'"), Some("'=='"), Some("'!='"), Some("'<='"), Some("'>='"), 
		Some("'<'"), Some("'>'"), Some("'true'"), Some("'false'"), Some("'null'")
	];
	pub const _SYMBOLIC_NAMES: [Option<&'static str>;93]  = [
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, Some("ID"), Some("NUMBER"), Some("STRING"), Some("LINE_COMMENT"), 
		Some("BLOCK_COMMENT"), Some("NL"), Some("WS")
	];
	lazy_static!{
	    static ref _shared_context_cache: Arc<PredictionContextCache> = Arc::new(PredictionContextCache::new());
		static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(_LITERAL_NAMES.iter(), _SYMBOLIC_NAMES.iter(), None));
	}


type BaseParserType<'input, I> =
	BaseParser<'input,EALParserExt<'input>, I, EALParserContextType , dyn EALListener<'input> + 'input >;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type EALTreeWalker<'input,'a> =
	ParseTreeWalker<'input, 'a, EALParserContextType , dyn EALListener<'input> + 'a>;

/// Parser for EAL grammar
pub struct EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	base:BaseParserType<'input,I>,
	interpreter:Rc<ParserATNSimulator>,
	_shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >,
}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn set_error_strategy(&mut self, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) {
        self.err_handler = strategy
    }

    pub fn with_strategy(input: I, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) -> Self {
		let interpreter = Rc::new(ParserATNSimulator::new(
			_ATN.clone(),
			_decision_to_DFA.clone(),
			_shared_context_cache.clone(),
		));
		Self {
			base: BaseParser::new_base_parser(
				input,
				Rc::clone(&interpreter),
				EALParserExt{
					_pd: Default::default(),
				}
			),
			interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }

}

type DynStrategy<'input,I> = Box<dyn ErrorStrategy<'input,BaseParserType<'input,I>> + 'input>;

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn new(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for EALParser
pub trait EALParserContext<'input>:
	for<'x> Listenable<dyn EALListener<'input> + 'x > + 
	for<'x> Visitable<dyn EALVisitor<'input> + 'x > + 
	ParserRuleContext<'input, TF=LocalTokenFactory<'input>, Ctx=EALParserContextType>
{}

antlr4rust::coerce_from!{ 'input : EALParserContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn EALParserContext<'input> + 'input
where
    T: EALVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn EALVisitor<'input> + 'x))
    }
}

impl<'input> EALParserContext<'input> for TerminalNode<'input,EALParserContextType> {}
impl<'input> EALParserContext<'input> for ErrorNode<'input,EALParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn EALParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn EALListener<'input> + 'input }

pub struct EALParserContextType;
antlr4rust::tid!{EALParserContextType}

impl<'input> ParserNodeType<'input> for EALParserContextType{
	type TF = LocalTokenFactory<'input>;
	type Type = dyn EALParserContext<'input> + 'input;
}

impl<'input, I> Deref for EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    type Target = BaseParserType<'input,I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct EALParserExt<'input>{
	_pd: PhantomData<&'input str>,
}

impl<'input> EALParserExt<'input>{
}
antlr4rust::tid! { EALParserExt<'a> }

impl<'input> TokenAware<'input> for EALParserExt<'input>{
	type TF = LocalTokenFactory<'input>;
}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> ParserRecog<'input, BaseParserType<'input,I>> for EALParserExt<'input>{}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> Actions<'input, BaseParserType<'input,I>> for EALParserExt<'input>{
	fn get_grammar_file_name(&self) -> & str{ "EAL.g4"}

   	fn get_rule_names(&self) -> &[& str] {&ruleNames}

   	fn get_vocabulary(&self) -> &dyn Vocabulary { &**VOCABULARY }
}
//------------------- program ----------------
pub type ProgramContextAll<'input> = ProgramContext<'input>;


pub type ProgramContext<'input> = BaseParserRuleContext<'input,ProgramContextExt<'input>>;

#[derive(Clone)]
pub struct ProgramContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ProgramContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ProgramContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_program(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_program(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ProgramContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_program(self);
	}
}

impl<'input> CustomRuleContext<'input> for ProgramContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_program }
	//fn type_rule_index() -> usize where Self: Sized { RULE_program }
}
antlr4rust::tid!{ProgramContextExt<'a>}

impl<'input> ProgramContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ProgramContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ProgramContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ProgramContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ProgramContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token STRING in current rule
fn STRING_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token STRING, starting from 0.
/// Returns `None` if number of children corresponding to token STRING is less or equal than `i`.
fn STRING(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, i)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_EOF, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ProgramContextAttrs<'input> for ProgramContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn program(&mut self,)
	-> Result<Rc<ProgramContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ProgramContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_program);
        let mut _localctx: Rc<ProgramContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(159);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(156);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(161);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(162);
			recog.base.match_token(EAL_T__0,&mut recog.err_handler)?;

			recog.base.set_state(163);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			recog.base.set_state(166);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__1 {
				{
				recog.base.set_state(164);
				recog.base.match_token(EAL_T__1,&mut recog.err_handler)?;

				recog.base.set_state(165);
				recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

				}
			}

			/*InvokeRule lineEnd*/
			recog.base.set_state(168);
			recog.lineEnd()?;

			recog.base.set_state(178);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(169);
				recog.declaration()?;

				recog.base.set_state(173);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(170);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(175);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(180);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(181);
			recog.base.match_token(EAL_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- declaration ----------------
pub type DeclarationContextAll<'input> = DeclarationContext<'input>;


pub type DeclarationContext<'input> = BaseParserRuleContext<'input,DeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct DeclarationContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for DeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for DeclarationContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_declaration(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_declaration(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for DeclarationContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_declaration(self);
	}
}

impl<'input> CustomRuleContext<'input> for DeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_declaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_declaration }
}
antlr4rust::tid!{DeclarationContextExt<'a>}

impl<'input> DeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<DeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,DeclarationContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait DeclarationContextAttrs<'input>: EALParserContext<'input> + BorrowMut<DeclarationContextExt<'input>>{

fn contextDecl(&self) -> Option<Rc<ContextDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn moduleDecl(&self) -> Option<Rc<ModuleDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn importDecl(&self) -> Option<Rc<ImportDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn environmentDecl(&self) -> Option<Rc<EnvironmentDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn toolDecl(&self) -> Option<Rc<ToolDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn evidenceDecl(&self) -> Option<Rc<EvidenceDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn assumptionDecl(&self) -> Option<Rc<AssumptionDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn reasoningDecl(&self) -> Option<Rc<ReasoningDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn claimDecl(&self) -> Option<Rc<ClaimDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn argumentDecl(&self) -> Option<Rc<ArgumentDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn objectionDecl(&self) -> Option<Rc<ObjectionDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn patternDecl(&self) -> Option<Rc<PatternDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn applicationDecl(&self) -> Option<Rc<ApplicationDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn patternGuard(&self) -> Option<Rc<PatternGuardContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn argumentationDirective(&self) -> Option<Rc<ArgumentationDirectiveContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> DeclarationContextAttrs<'input> for DeclarationContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn declaration(&mut self,)
	-> Result<Rc<DeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = DeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_declaration);
        let mut _localctx: Rc<DeclarationContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(198);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__7 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule contextDecl*/
					recog.base.set_state(183);
					recog.contextDecl()?;

					}
				}

			EAL_T__2 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule moduleDecl*/
					recog.base.set_state(184);
					recog.moduleDecl()?;

					}
				}

			EAL_T__5 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule importDecl*/
					recog.base.set_state(185);
					recog.importDecl()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule environmentDecl*/
					recog.base.set_state(186);
					recog.environmentDecl()?;

					}
				}

			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule toolDecl*/
					recog.base.set_state(187);
					recog.toolDecl()?;

					}
				}

			EAL_T__24 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule evidenceDecl*/
					recog.base.set_state(188);
					recog.evidenceDecl()?;

					}
				}

			EAL_T__28 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule assumptionDecl*/
					recog.base.set_state(189);
					recog.assumptionDecl()?;

					}
				}

			EAL_T__30 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule reasoningDecl*/
					recog.base.set_state(190);
					recog.reasoningDecl()?;

					}
				}

			EAL_T__37 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					/*InvokeRule claimDecl*/
					recog.base.set_state(191);
					recog.claimDecl()?;

					}
				}

			EAL_T__40 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					/*InvokeRule argumentDecl*/
					recog.base.set_state(192);
					recog.argumentDecl()?;

					}
				}

			EAL_T__57 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					/*InvokeRule objectionDecl*/
					recog.base.set_state(193);
					recog.objectionDecl()?;

					}
				}

			EAL_T__49 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					/*InvokeRule patternDecl*/
					recog.base.set_state(194);
					recog.patternDecl()?;

					}
				}

			EAL_T__54 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					/*InvokeRule applicationDecl*/
					recog.base.set_state(195);
					recog.applicationDecl()?;

					}
				}

			EAL_T__55 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					/*InvokeRule patternGuard*/
					recog.base.set_state(196);
					recog.patternGuard()?;

					}
				}

			EAL_T__59 |EAL_T__60 |EAL_T__61 |EAL_T__62 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					/*InvokeRule argumentationDirective*/
					recog.base.set_state(197);
					recog.argumentationDirective()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- moduleDecl ----------------
pub type ModuleDeclContextAll<'input> = ModuleDeclContext<'input>;


pub type ModuleDeclContext<'input> = BaseParserRuleContext<'input,ModuleDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ModuleDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ModuleDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ModuleDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_moduleDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_moduleDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ModuleDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_moduleDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ModuleDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_moduleDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_moduleDecl }
}
antlr4rust::tid!{ModuleDeclContextExt<'a>}

impl<'input> ModuleDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ModuleDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ModuleDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ModuleDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ModuleDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ModuleDeclContextAttrs<'input> for ModuleDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn moduleDecl(&mut self,)
	-> Result<Rc<ModuleDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ModuleDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_moduleDecl);
        let mut _localctx: Rc<ModuleDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(200);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(201);
			recog.qualifiedName()?;

			recog.base.set_state(202);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(206);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(203);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(208);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(218);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(209);
				recog.declaration()?;

				recog.base.set_state(213);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(210);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(215);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(220);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(221);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- importDecl ----------------
pub type ImportDeclContextAll<'input> = ImportDeclContext<'input>;


pub type ImportDeclContext<'input> = BaseParserRuleContext<'input,ImportDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ImportDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ImportDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ImportDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_importDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_importDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ImportDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_importDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ImportDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_importDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_importDecl }
}
antlr4rust::tid!{ImportDeclContextExt<'a>}

impl<'input> ImportDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ImportDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ImportDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ImportDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ImportDeclContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ImportDeclContextAttrs<'input> for ImportDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn importDecl(&mut self,)
	-> Result<Rc<ImportDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ImportDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_importDecl);
        let mut _localctx: Rc<ImportDeclContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(223);
			recog.base.match_token(EAL_T__5,&mut recog.err_handler)?;

			recog.base.set_state(224);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			recog.base.set_state(225);
			recog.base.match_token(EAL_T__6,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(226);
			recog.identifier()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(227);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- contextDecl ----------------
pub type ContextDeclContextAll<'input> = ContextDeclContext<'input>;


pub type ContextDeclContext<'input> = BaseParserRuleContext<'input,ContextDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ContextDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ContextDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ContextDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_contextDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_contextDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ContextDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_contextDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ContextDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_contextDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_contextDecl }
}
antlr4rust::tid!{ContextDeclContextExt<'a>}

impl<'input> ContextDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ContextDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ContextDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ContextDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ContextDeclContextExt<'input>>{

fn contextAttribute_all(&self) ->  Vec<Rc<ContextAttributeContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn contextAttribute(&self, i: usize) -> Option<Rc<ContextAttributeContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ContextDeclContextAttrs<'input> for ContextDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn contextDecl(&mut self,)
	-> Result<Rc<ContextDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ContextDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_contextDecl);
        let mut _localctx: Rc<ContextDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(229);
			recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

			/*InvokeRule contextAttribute*/
			recog.base.set_state(230);
			recog.contextAttribute()?;

			recog.base.set_state(235);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__8 {
				{
				{
				recog.base.set_state(231);
				recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

				/*InvokeRule contextAttribute*/
				recog.base.set_state(232);
				recog.contextAttribute()?;

				}
				}
				recog.base.set_state(237);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(238);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(242);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(239);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(244);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(254);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(245);
				recog.declaration()?;

				recog.base.set_state(249);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(246);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(251);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(256);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(257);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- contextAttribute ----------------
pub type ContextAttributeContextAll<'input> = ContextAttributeContext<'input>;


pub type ContextAttributeContext<'input> = BaseParserRuleContext<'input,ContextAttributeContextExt<'input>>;

#[derive(Clone)]
pub struct ContextAttributeContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ContextAttributeContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ContextAttributeContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_contextAttribute(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_contextAttribute(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ContextAttributeContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_contextAttribute(self);
	}
}

impl<'input> CustomRuleContext<'input> for ContextAttributeContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_contextAttribute }
	//fn type_rule_index() -> usize where Self: Sized { RULE_contextAttribute }
}
antlr4rust::tid!{ContextAttributeContextExt<'a>}

impl<'input> ContextAttributeContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ContextAttributeContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ContextAttributeContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ContextAttributeContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ContextAttributeContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn signedNumber(&self) -> Option<Rc<SignedNumberContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}

}

impl<'input> ContextAttributeContextAttrs<'input> for ContextAttributeContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn contextAttribute(&mut self,)
	-> Result<Rc<ContextAttributeContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ContextAttributeContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_contextAttribute);
        let mut _localctx: Rc<ContextAttributeContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(289);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(259);
					recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(260);
					recog.qualifiedName()?;

					}
				}

			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(261);
					recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(262);
					recog.qualifiedName()?;

					}
				}

			EAL_T__11 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(263);
					recog.base.match_token(EAL_T__11,&mut recog.err_handler)?;

					/*InvokeRule signedNumber*/
					recog.base.set_state(264);
					recog.signedNumber()?;

					}
				}

			EAL_T__12 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(265);
					recog.base.match_token(EAL_T__12,&mut recog.err_handler)?;

					/*InvokeRule identifier*/
					recog.base.set_state(266);
					recog.identifier()?;

					}
				}

			EAL_T__13 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(267);
					recog.base.match_token(EAL_T__13,&mut recog.err_handler)?;

					/*InvokeRule jsonValue*/
					recog.base.set_state(268);
					recog.jsonValue()?;

					}
				}

			EAL_T__14 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(269);
					recog.base.match_token(EAL_T__14,&mut recog.err_handler)?;

					recog.base.set_state(270);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__15 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					recog.base.set_state(271);
					recog.base.match_token(EAL_T__15,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(272);
					recog.qualifiedName()?;

					}
				}

			EAL_T__16 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					recog.base.set_state(273);
					recog.base.match_token(EAL_T__16,&mut recog.err_handler)?;

					recog.base.set_state(274);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__17 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					recog.base.set_state(275);
					recog.base.match_token(EAL_T__17,&mut recog.err_handler)?;

					recog.base.set_state(276);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__18 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					recog.base.set_state(277);
					recog.base.match_token(EAL_T__18,&mut recog.err_handler)?;

					recog.base.set_state(278);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__19 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					recog.base.set_state(279);
					recog.base.match_token(EAL_T__19,&mut recog.err_handler)?;

					recog.base.set_state(280);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__20 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					recog.base.set_state(281);
					recog.base.match_token(EAL_T__20,&mut recog.err_handler)?;

					recog.base.set_state(282);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					recog.base.set_state(283);
					recog.base.match_token(EAL_T__21,&mut recog.err_handler)?;

					recog.base.set_state(284);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__22 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					recog.base.set_state(285);
					recog.base.match_token(EAL_T__22,&mut recog.err_handler)?;

					recog.base.set_state(286);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__23 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					recog.base.set_state(287);
					recog.base.match_token(EAL_T__23,&mut recog.err_handler)?;

					/*InvokeRule jsonValue*/
					recog.base.set_state(288);
					recog.jsonValue()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- environmentDecl ----------------
pub type EnvironmentDeclContextAll<'input> = EnvironmentDeclContext<'input>;


pub type EnvironmentDeclContext<'input> = BaseParserRuleContext<'input,EnvironmentDeclContextExt<'input>>;

#[derive(Clone)]
pub struct EnvironmentDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for EnvironmentDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for EnvironmentDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_environmentDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_environmentDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for EnvironmentDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_environmentDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for EnvironmentDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_environmentDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_environmentDecl }
}
antlr4rust::tid!{EnvironmentDeclContextExt<'a>}

impl<'input> EnvironmentDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EnvironmentDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EnvironmentDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait EnvironmentDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<EnvironmentDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn predicate_all(&self) ->  Vec<Rc<PredicateContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn predicate(&self, i: usize) -> Option<Rc<PredicateContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> EnvironmentDeclContextAttrs<'input> for EnvironmentDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn environmentDecl(&mut self,)
	-> Result<Rc<EnvironmentDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EnvironmentDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 12, RULE_environmentDecl);
        let mut _localctx: Rc<EnvironmentDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(291);
			recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(292);
			recog.qualifiedName()?;

			recog.base.set_state(293);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(297);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(294);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(299);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(301); 
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			loop {
				{
				{
				/*InvokeRule predicate*/
				recog.base.set_state(300);
				recog.predicate()?;

				}
				}
				recog.base.set_state(303); 
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				if !(_la==EAL_T__67) {break}
			}
			recog.base.set_state(305);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- toolDecl ----------------
pub type ToolDeclContextAll<'input> = ToolDeclContext<'input>;


pub type ToolDeclContext<'input> = BaseParserRuleContext<'input,ToolDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ToolDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ToolDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ToolDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_toolDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_toolDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ToolDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_toolDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ToolDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_toolDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_toolDecl }
}
antlr4rust::tid!{ToolDeclContextExt<'a>}

impl<'input> ToolDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ToolDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ToolDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ToolDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ToolDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn versionField_all(&self) ->  Vec<Rc<VersionFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn versionField(&self, i: usize) -> Option<Rc<VersionFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ToolDeclContextAttrs<'input> for ToolDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn toolDecl(&mut self,)
	-> Result<Rc<ToolDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ToolDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_toolDecl);
        let mut _localctx: Rc<ToolDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(307);
			recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(308);
			recog.qualifiedName()?;

			recog.base.set_state(309);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(313);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(310);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(315);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(319);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__14 {
				{
				{
				/*InvokeRule versionField*/
				recog.base.set_state(316);
				recog.versionField()?;

				}
				}
				recog.base.set_state(321);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(322);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- versionField ----------------
pub type VersionFieldContextAll<'input> = VersionFieldContext<'input>;


pub type VersionFieldContext<'input> = BaseParserRuleContext<'input,VersionFieldContextExt<'input>>;

#[derive(Clone)]
pub struct VersionFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for VersionFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for VersionFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_versionField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_versionField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for VersionFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_versionField(self);
	}
}

impl<'input> CustomRuleContext<'input> for VersionFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_versionField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_versionField }
}
antlr4rust::tid!{VersionFieldContextExt<'a>}

impl<'input> VersionFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<VersionFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,VersionFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait VersionFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<VersionFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> VersionFieldContextAttrs<'input> for VersionFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn versionField(&mut self,)
	-> Result<Rc<VersionFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = VersionFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 16, RULE_versionField);
        let mut _localctx: Rc<VersionFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(324);
			recog.base.match_token(EAL_T__14,&mut recog.err_handler)?;

			recog.base.set_state(325);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(326);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- evidenceDecl ----------------
pub type EvidenceDeclContextAll<'input> = EvidenceDeclContext<'input>;


pub type EvidenceDeclContext<'input> = BaseParserRuleContext<'input,EvidenceDeclContextExt<'input>>;

#[derive(Clone)]
pub struct EvidenceDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for EvidenceDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for EvidenceDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_evidenceDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_evidenceDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for EvidenceDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_evidenceDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for EvidenceDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_evidenceDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_evidenceDecl }
}
antlr4rust::tid!{EvidenceDeclContextExt<'a>}

impl<'input> EvidenceDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EvidenceDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EvidenceDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait EvidenceDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<EvidenceDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn evidenceField_all(&self) ->  Vec<Rc<EvidenceFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn evidenceField(&self, i: usize) -> Option<Rc<EvidenceFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> EvidenceDeclContextAttrs<'input> for EvidenceDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn evidenceDecl(&mut self,)
	-> Result<Rc<EvidenceDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EvidenceDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_evidenceDecl);
        let mut _localctx: Rc<EvidenceDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(328);
			recog.base.match_token(EAL_T__24,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(329);
			recog.qualifiedName()?;

			recog.base.set_state(330);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(334);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(331);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(336);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(340);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 335576064) != 0) || _la==EAL_T__67 {
				{
				{
				/*InvokeRule evidenceField*/
				recog.base.set_state(337);
				recog.evidenceField()?;

				}
				}
				recog.base.set_state(342);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(343);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- evidenceField ----------------
pub type EvidenceFieldContextAll<'input> = EvidenceFieldContext<'input>;


pub type EvidenceFieldContext<'input> = BaseParserRuleContext<'input,EvidenceFieldContextExt<'input>>;

#[derive(Clone)]
pub struct EvidenceFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for EvidenceFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for EvidenceFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_evidenceField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_evidenceField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for EvidenceFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_evidenceField(self);
	}
}

impl<'input> CustomRuleContext<'input> for EvidenceFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_evidenceField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_evidenceField }
}
antlr4rust::tid!{EvidenceFieldContextExt<'a>}

impl<'input> EvidenceFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EvidenceFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EvidenceFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait EvidenceFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<EvidenceFieldContextExt<'input>>{

fn toolField(&self) -> Option<Rc<ToolFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn deriveField(&self) -> Option<Rc<DeriveFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn kindField(&self) -> Option<Rc<KindFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn environmentField(&self) -> Option<Rc<EnvironmentFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn maxAgeField(&self) -> Option<Rc<MaxAgeFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn inputField(&self) -> Option<Rc<InputFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn inputBindingField(&self) -> Option<Rc<InputBindingFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn predicate(&self) -> Option<Rc<PredicateContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> EvidenceFieldContextAttrs<'input> for EvidenceFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn evidenceField(&mut self,)
	-> Result<Rc<EvidenceFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EvidenceFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_evidenceField);
        let mut _localctx: Rc<EvidenceFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(353);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule toolField*/
					recog.base.set_state(345);
					recog.toolField()?;

					}
				}

			EAL_T__27 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule deriveField*/
					recog.base.set_state(346);
					recog.deriveField()?;

					}
				}

			EAL_T__12 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule kindField*/
					recog.base.set_state(347);
					recog.kindField()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(348);
					recog.environmentField()?;

					}
				}

			EAL_T__11 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule maxAgeField*/
					recog.base.set_state(349);
					recog.maxAgeField()?;

					}
				}

			EAL_T__13 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule inputField*/
					recog.base.set_state(350);
					recog.inputField()?;

					}
				}

			EAL_T__25 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule inputBindingField*/
					recog.base.set_state(351);
					recog.inputBindingField()?;

					}
				}

			EAL_T__67 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule predicate*/
					recog.base.set_state(352);
					recog.predicate()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- toolField ----------------
pub type ToolFieldContextAll<'input> = ToolFieldContext<'input>;


pub type ToolFieldContext<'input> = BaseParserRuleContext<'input,ToolFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ToolFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ToolFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ToolFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_toolField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_toolField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ToolFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_toolField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ToolFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_toolField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_toolField }
}
antlr4rust::tid!{ToolFieldContextExt<'a>}

impl<'input> ToolFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ToolFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ToolFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ToolFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ToolFieldContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ToolFieldContextAttrs<'input> for ToolFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn toolField(&mut self,)
	-> Result<Rc<ToolFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ToolFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_toolField);
        let mut _localctx: Rc<ToolFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(355);
			recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(356);
			recog.qualifiedName()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(357);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- kindField ----------------
pub type KindFieldContextAll<'input> = KindFieldContext<'input>;


pub type KindFieldContext<'input> = BaseParserRuleContext<'input,KindFieldContextExt<'input>>;

#[derive(Clone)]
pub struct KindFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for KindFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for KindFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_kindField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_kindField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for KindFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_kindField(self);
	}
}

impl<'input> CustomRuleContext<'input> for KindFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_kindField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_kindField }
}
antlr4rust::tid!{KindFieldContextExt<'a>}

impl<'input> KindFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<KindFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,KindFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait KindFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<KindFieldContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> KindFieldContextAttrs<'input> for KindFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn kindField(&mut self,)
	-> Result<Rc<KindFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = KindFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 24, RULE_kindField);
        let mut _localctx: Rc<KindFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(359);
			recog.base.match_token(EAL_T__12,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(360);
			recog.identifier()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(361);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- environmentField ----------------
pub type EnvironmentFieldContextAll<'input> = EnvironmentFieldContext<'input>;


pub type EnvironmentFieldContext<'input> = BaseParserRuleContext<'input,EnvironmentFieldContextExt<'input>>;

#[derive(Clone)]
pub struct EnvironmentFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for EnvironmentFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for EnvironmentFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_environmentField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_environmentField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for EnvironmentFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_environmentField(self);
	}
}

impl<'input> CustomRuleContext<'input> for EnvironmentFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_environmentField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_environmentField }
}
antlr4rust::tid!{EnvironmentFieldContextExt<'a>}

impl<'input> EnvironmentFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<EnvironmentFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,EnvironmentFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait EnvironmentFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<EnvironmentFieldContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> EnvironmentFieldContextAttrs<'input> for EnvironmentFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn environmentField(&mut self,)
	-> Result<Rc<EnvironmentFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = EnvironmentFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 26, RULE_environmentField);
        let mut _localctx: Rc<EnvironmentFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(363);
			recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(364);
			recog.qualifiedName()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(365);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- maxAgeField ----------------
pub type MaxAgeFieldContextAll<'input> = MaxAgeFieldContext<'input>;


pub type MaxAgeFieldContext<'input> = BaseParserRuleContext<'input,MaxAgeFieldContextExt<'input>>;

#[derive(Clone)]
pub struct MaxAgeFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for MaxAgeFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for MaxAgeFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_maxAgeField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_maxAgeField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for MaxAgeFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_maxAgeField(self);
	}
}

impl<'input> CustomRuleContext<'input> for MaxAgeFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_maxAgeField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_maxAgeField }
}
antlr4rust::tid!{MaxAgeFieldContextExt<'a>}

impl<'input> MaxAgeFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MaxAgeFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MaxAgeFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait MaxAgeFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<MaxAgeFieldContextExt<'input>>{

fn signedNumber(&self) -> Option<Rc<SignedNumberContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> MaxAgeFieldContextAttrs<'input> for MaxAgeFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn maxAgeField(&mut self,)
	-> Result<Rc<MaxAgeFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MaxAgeFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 28, RULE_maxAgeField);
        let mut _localctx: Rc<MaxAgeFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(367);
			recog.base.match_token(EAL_T__11,&mut recog.err_handler)?;

			/*InvokeRule signedNumber*/
			recog.base.set_state(368);
			recog.signedNumber()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(369);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- inputField ----------------
pub type InputFieldContextAll<'input> = InputFieldContext<'input>;


pub type InputFieldContext<'input> = BaseParserRuleContext<'input,InputFieldContextExt<'input>>;

#[derive(Clone)]
pub struct InputFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for InputFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for InputFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_inputField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_inputField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for InputFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_inputField(self);
	}
}

impl<'input> CustomRuleContext<'input> for InputFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_inputField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_inputField }
}
antlr4rust::tid!{InputFieldContextExt<'a>}

impl<'input> InputFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<InputFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,InputFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait InputFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<InputFieldContextExt<'input>>{

fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> InputFieldContextAttrs<'input> for InputFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn inputField(&mut self,)
	-> Result<Rc<InputFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = InputFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 30, RULE_inputField);
        let mut _localctx: Rc<InputFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(371);
			recog.base.match_token(EAL_T__13,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(372);
			recog.jsonValue()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(373);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- inputBindingField ----------------
pub type InputBindingFieldContextAll<'input> = InputBindingFieldContext<'input>;


pub type InputBindingFieldContext<'input> = BaseParserRuleContext<'input,InputBindingFieldContextExt<'input>>;

#[derive(Clone)]
pub struct InputBindingFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for InputBindingFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for InputBindingFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_inputBindingField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_inputBindingField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for InputBindingFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_inputBindingField(self);
	}
}

impl<'input> CustomRuleContext<'input> for InputBindingFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_inputBindingField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_inputBindingField }
}
antlr4rust::tid!{InputBindingFieldContextExt<'a>}

impl<'input> InputBindingFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<InputBindingFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,InputBindingFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait InputBindingFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<InputBindingFieldContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token STRING in current rule
fn STRING_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token STRING, starting from 0.
/// Returns `None` if number of children corresponding to token STRING is less or equal than `i`.
fn STRING(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, i)
}
fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> InputBindingFieldContextAttrs<'input> for InputBindingFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn inputBindingField(&mut self,)
	-> Result<Rc<InputBindingFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = InputBindingFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 32, RULE_inputBindingField);
        let mut _localctx: Rc<InputBindingFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(375);
			recog.base.match_token(EAL_T__25,&mut recog.err_handler)?;

			recog.base.set_state(376);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			recog.base.set_state(377);
			recog.base.match_token(EAL_T__26,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(378);
			recog.qualifiedName()?;

			recog.base.set_state(379);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(380);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- deriveField ----------------
pub type DeriveFieldContextAll<'input> = DeriveFieldContext<'input>;


pub type DeriveFieldContext<'input> = BaseParserRuleContext<'input,DeriveFieldContextExt<'input>>;

#[derive(Clone)]
pub struct DeriveFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for DeriveFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for DeriveFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_deriveField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_deriveField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for DeriveFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_deriveField(self);
	}
}

impl<'input> CustomRuleContext<'input> for DeriveFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_deriveField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_deriveField }
}
antlr4rust::tid!{DeriveFieldContextExt<'a>}

impl<'input> DeriveFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<DeriveFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,DeriveFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait DeriveFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<DeriveFieldContextExt<'input>>{

fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> DeriveFieldContextAttrs<'input> for DeriveFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn deriveField(&mut self,)
	-> Result<Rc<DeriveFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = DeriveFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 34, RULE_deriveField);
        let mut _localctx: Rc<DeriveFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(382);
			recog.base.match_token(EAL_T__27,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(383);
			recog.expression()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(384);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- assumptionDecl ----------------
pub type AssumptionDeclContextAll<'input> = AssumptionDeclContext<'input>;


pub type AssumptionDeclContext<'input> = BaseParserRuleContext<'input,AssumptionDeclContextExt<'input>>;

#[derive(Clone)]
pub struct AssumptionDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for AssumptionDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for AssumptionDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_assumptionDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_assumptionDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for AssumptionDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_assumptionDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for AssumptionDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_assumptionDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_assumptionDecl }
}
antlr4rust::tid!{AssumptionDeclContextExt<'a>}

impl<'input> AssumptionDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<AssumptionDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AssumptionDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AssumptionDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<AssumptionDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn assumptionField_all(&self) ->  Vec<Rc<AssumptionFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn assumptionField(&self, i: usize) -> Option<Rc<AssumptionFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> AssumptionDeclContextAttrs<'input> for AssumptionDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn assumptionDecl(&mut self,)
	-> Result<Rc<AssumptionDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AssumptionDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 36, RULE_assumptionDecl);
        let mut _localctx: Rc<AssumptionDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(386);
			recog.base.match_token(EAL_T__28,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(387);
			recog.qualifiedName()?;

			recog.base.set_state(388);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(392);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(389);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(394);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(398);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1086391296) != 0) {
				{
				{
				/*InvokeRule assumptionField*/
				recog.base.set_state(395);
				recog.assumptionField()?;

				}
				}
				recog.base.set_state(400);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(401);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- assumptionField ----------------
pub type AssumptionFieldContextAll<'input> = AssumptionFieldContext<'input>;


pub type AssumptionFieldContext<'input> = BaseParserRuleContext<'input,AssumptionFieldContextExt<'input>>;

#[derive(Clone)]
pub struct AssumptionFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for AssumptionFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for AssumptionFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_assumptionField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_assumptionField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for AssumptionFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_assumptionField(self);
	}
}

impl<'input> CustomRuleContext<'input> for AssumptionFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_assumptionField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_assumptionField }
}
antlr4rust::tid!{AssumptionFieldContextExt<'a>}

impl<'input> AssumptionFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<AssumptionFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AssumptionFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AssumptionFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<AssumptionFieldContextExt<'input>>{

fn statementField(&self) -> Option<Rc<StatementFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn environmentField(&self) -> Option<Rc<EnvironmentFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn validateField(&self) -> Option<Rc<ValidateFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn validFromField(&self) -> Option<Rc<ValidFromFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn validUntilField(&self) -> Option<Rc<ValidUntilFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> AssumptionFieldContextAttrs<'input> for AssumptionFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn assumptionField(&mut self,)
	-> Result<Rc<AssumptionFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AssumptionFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 38, RULE_assumptionField);
        let mut _localctx: Rc<AssumptionFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(408);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__29 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule statementField*/
					recog.base.set_state(403);
					recog.statementField()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(404);
					recog.environmentField()?;

					}
				}

			EAL_T__15 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule validateField*/
					recog.base.set_state(405);
					recog.validateField()?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule validFromField*/
					recog.base.set_state(406);
					recog.validFromField()?;

					}
				}

			EAL_T__22 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule validUntilField*/
					recog.base.set_state(407);
					recog.validUntilField()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- statementField ----------------
pub type StatementFieldContextAll<'input> = StatementFieldContext<'input>;


pub type StatementFieldContext<'input> = BaseParserRuleContext<'input,StatementFieldContextExt<'input>>;

#[derive(Clone)]
pub struct StatementFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for StatementFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for StatementFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_statementField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_statementField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for StatementFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_statementField(self);
	}
}

impl<'input> CustomRuleContext<'input> for StatementFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_statementField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_statementField }
}
antlr4rust::tid!{StatementFieldContextExt<'a>}

impl<'input> StatementFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<StatementFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,StatementFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait StatementFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<StatementFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> StatementFieldContextAttrs<'input> for StatementFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn statementField(&mut self,)
	-> Result<Rc<StatementFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = StatementFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 40, RULE_statementField);
        let mut _localctx: Rc<StatementFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(410);
			recog.base.match_token(EAL_T__29,&mut recog.err_handler)?;

			recog.base.set_state(411);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(412);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- validateField ----------------
pub type ValidateFieldContextAll<'input> = ValidateFieldContext<'input>;


pub type ValidateFieldContext<'input> = BaseParserRuleContext<'input,ValidateFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ValidateFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ValidateFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ValidateFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_validateField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_validateField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ValidateFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_validateField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ValidateFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_validateField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_validateField }
}
antlr4rust::tid!{ValidateFieldContextExt<'a>}

impl<'input> ValidateFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ValidateFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ValidateFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ValidateFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ValidateFieldContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ValidateFieldContextAttrs<'input> for ValidateFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn validateField(&mut self,)
	-> Result<Rc<ValidateFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ValidateFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 42, RULE_validateField);
        let mut _localctx: Rc<ValidateFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(414);
			recog.base.match_token(EAL_T__15,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(415);
			recog.qualifiedName()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(416);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- validFromField ----------------
pub type ValidFromFieldContextAll<'input> = ValidFromFieldContext<'input>;


pub type ValidFromFieldContext<'input> = BaseParserRuleContext<'input,ValidFromFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ValidFromFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ValidFromFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ValidFromFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_validFromField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_validFromField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ValidFromFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_validFromField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ValidFromFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_validFromField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_validFromField }
}
antlr4rust::tid!{ValidFromFieldContextExt<'a>}

impl<'input> ValidFromFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ValidFromFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ValidFromFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ValidFromFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ValidFromFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ValidFromFieldContextAttrs<'input> for ValidFromFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn validFromField(&mut self,)
	-> Result<Rc<ValidFromFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ValidFromFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 44, RULE_validFromField);
        let mut _localctx: Rc<ValidFromFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(418);
			recog.base.match_token(EAL_T__21,&mut recog.err_handler)?;

			recog.base.set_state(419);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(420);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- validUntilField ----------------
pub type ValidUntilFieldContextAll<'input> = ValidUntilFieldContext<'input>;


pub type ValidUntilFieldContext<'input> = BaseParserRuleContext<'input,ValidUntilFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ValidUntilFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ValidUntilFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ValidUntilFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_validUntilField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_validUntilField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ValidUntilFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_validUntilField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ValidUntilFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_validUntilField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_validUntilField }
}
antlr4rust::tid!{ValidUntilFieldContextExt<'a>}

impl<'input> ValidUntilFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ValidUntilFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ValidUntilFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ValidUntilFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ValidUntilFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ValidUntilFieldContextAttrs<'input> for ValidUntilFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn validUntilField(&mut self,)
	-> Result<Rc<ValidUntilFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ValidUntilFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 46, RULE_validUntilField);
        let mut _localctx: Rc<ValidUntilFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(422);
			recog.base.match_token(EAL_T__22,&mut recog.err_handler)?;

			recog.base.set_state(423);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(424);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- reasoningDecl ----------------
pub type ReasoningDeclContextAll<'input> = ReasoningDeclContext<'input>;


pub type ReasoningDeclContext<'input> = BaseParserRuleContext<'input,ReasoningDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ReasoningDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ReasoningDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ReasoningDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_reasoningDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_reasoningDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ReasoningDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_reasoningDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ReasoningDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_reasoningDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_reasoningDecl }
}
antlr4rust::tid!{ReasoningDeclContextExt<'a>}

impl<'input> ReasoningDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ReasoningDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ReasoningDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ReasoningDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ReasoningDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn reasoningField_all(&self) ->  Vec<Rc<ReasoningFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reasoningField(&self, i: usize) -> Option<Rc<ReasoningFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ReasoningDeclContextAttrs<'input> for ReasoningDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn reasoningDecl(&mut self,)
	-> Result<Rc<ReasoningDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ReasoningDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 48, RULE_reasoningDecl);
        let mut _localctx: Rc<ReasoningDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(426);
			recog.base.match_token(EAL_T__30,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(427);
			recog.qualifiedName()?;

			recog.base.set_state(428);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(432);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(429);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(434);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(438);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 17)) & !0x3f) == 0 && ((1usize << (_la - 17)) & 229377) != 0) || _la==EAL_T__67 {
				{
				{
				/*InvokeRule reasoningField*/
				recog.base.set_state(435);
				recog.reasoningField()?;

				}
				}
				recog.base.set_state(440);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(441);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- reasoningField ----------------
pub type ReasoningFieldContextAll<'input> = ReasoningFieldContext<'input>;


pub type ReasoningFieldContext<'input> = BaseParserRuleContext<'input,ReasoningFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ReasoningFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ReasoningFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ReasoningFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_reasoningField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_reasoningField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ReasoningFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_reasoningField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ReasoningFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_reasoningField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_reasoningField }
}
antlr4rust::tid!{ReasoningFieldContextExt<'a>}

impl<'input> ReasoningFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ReasoningFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ReasoningFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ReasoningFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ReasoningFieldContextExt<'input>>{

fn methodField(&self) -> Option<Rc<MethodFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn rationaleField(&self) -> Option<Rc<RationaleFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn backingField(&self) -> Option<Rc<BackingFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn transferField(&self) -> Option<Rc<TransferFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn predicate(&self) -> Option<Rc<PredicateContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ReasoningFieldContextAttrs<'input> for ReasoningFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn reasoningField(&mut self,)
	-> Result<Rc<ReasoningFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ReasoningFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 50, RULE_reasoningField);
        let mut _localctx: Rc<ReasoningFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(448);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__16 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule methodField*/
					recog.base.set_state(443);
					recog.methodField()?;

					}
				}

			EAL_T__31 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule rationaleField*/
					recog.base.set_state(444);
					recog.rationaleField()?;

					}
				}

			EAL_T__32 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule backingField*/
					recog.base.set_state(445);
					recog.backingField()?;

					}
				}

			EAL_T__33 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule transferField*/
					recog.base.set_state(446);
					recog.transferField()?;

					}
				}

			EAL_T__67 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule predicate*/
					recog.base.set_state(447);
					recog.predicate()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- methodField ----------------
pub type MethodFieldContextAll<'input> = MethodFieldContext<'input>;


pub type MethodFieldContext<'input> = BaseParserRuleContext<'input,MethodFieldContextExt<'input>>;

#[derive(Clone)]
pub struct MethodFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for MethodFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for MethodFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_methodField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_methodField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for MethodFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_methodField(self);
	}
}

impl<'input> CustomRuleContext<'input> for MethodFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_methodField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_methodField }
}
antlr4rust::tid!{MethodFieldContextExt<'a>}

impl<'input> MethodFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MethodFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MethodFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait MethodFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<MethodFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> MethodFieldContextAttrs<'input> for MethodFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn methodField(&mut self,)
	-> Result<Rc<MethodFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MethodFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 52, RULE_methodField);
        let mut _localctx: Rc<MethodFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(450);
			recog.base.match_token(EAL_T__16,&mut recog.err_handler)?;

			recog.base.set_state(451);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(452);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- rationaleField ----------------
pub type RationaleFieldContextAll<'input> = RationaleFieldContext<'input>;


pub type RationaleFieldContext<'input> = BaseParserRuleContext<'input,RationaleFieldContextExt<'input>>;

#[derive(Clone)]
pub struct RationaleFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for RationaleFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for RationaleFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_rationaleField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_rationaleField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for RationaleFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_rationaleField(self);
	}
}

impl<'input> CustomRuleContext<'input> for RationaleFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_rationaleField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_rationaleField }
}
antlr4rust::tid!{RationaleFieldContextExt<'a>}

impl<'input> RationaleFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<RationaleFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,RationaleFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait RationaleFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<RationaleFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> RationaleFieldContextAttrs<'input> for RationaleFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn rationaleField(&mut self,)
	-> Result<Rc<RationaleFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = RationaleFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 54, RULE_rationaleField);
        let mut _localctx: Rc<RationaleFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(454);
			recog.base.match_token(EAL_T__31,&mut recog.err_handler)?;

			recog.base.set_state(455);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(456);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- backingField ----------------
pub type BackingFieldContextAll<'input> = BackingFieldContext<'input>;


pub type BackingFieldContext<'input> = BaseParserRuleContext<'input,BackingFieldContextExt<'input>>;

#[derive(Clone)]
pub struct BackingFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for BackingFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for BackingFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_backingField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_backingField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for BackingFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_backingField(self);
	}
}

impl<'input> CustomRuleContext<'input> for BackingFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_backingField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_backingField }
}
antlr4rust::tid!{BackingFieldContextExt<'a>}

impl<'input> BackingFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<BackingFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BackingFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BackingFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<BackingFieldContextExt<'input>>{

fn referenceList(&self) -> Option<Rc<ReferenceListContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> BackingFieldContextAttrs<'input> for BackingFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn backingField(&mut self,)
	-> Result<Rc<BackingFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BackingFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 56, RULE_backingField);
        let mut _localctx: Rc<BackingFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(458);
			recog.base.match_token(EAL_T__32,&mut recog.err_handler)?;

			/*InvokeRule referenceList*/
			recog.base.set_state(459);
			recog.referenceList()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(460);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- transferField ----------------
pub type TransferFieldContextAll<'input> = TransferFieldContext<'input>;


pub type TransferFieldContext<'input> = BaseParserRuleContext<'input,TransferFieldContextExt<'input>>;

#[derive(Clone)]
pub struct TransferFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for TransferFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for TransferFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_transferField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_transferField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for TransferFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_transferField(self);
	}
}

impl<'input> CustomRuleContext<'input> for TransferFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_transferField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_transferField }
}
antlr4rust::tid!{TransferFieldContextExt<'a>}

impl<'input> TransferFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<TransferFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,TransferFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait TransferFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<TransferFieldContextExt<'input>>{

fn qualifiedName_all(&self) ->  Vec<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn qualifiedName(&self, i: usize) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> TransferFieldContextAttrs<'input> for TransferFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn transferField(&mut self,)
	-> Result<Rc<TransferFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = TransferFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 58, RULE_transferField);
        let mut _localctx: Rc<TransferFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(462);
			recog.base.match_token(EAL_T__33,&mut recog.err_handler)?;

			recog.base.set_state(463);
			recog.base.match_token(EAL_T__26,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(464);
			recog.qualifiedName()?;

			recog.base.set_state(465);
			recog.base.match_token(EAL_T__34,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(466);
			recog.qualifiedName()?;

			recog.base.set_state(467);
			recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(468);
			recog.qualifiedName()?;

			recog.base.set_state(469);
			recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

			recog.base.set_state(470);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(471);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- claimDecl ----------------
pub type ClaimDeclContextAll<'input> = ClaimDeclContext<'input>;


pub type ClaimDeclContext<'input> = BaseParserRuleContext<'input,ClaimDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ClaimDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ClaimDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ClaimDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_claimDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_claimDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ClaimDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_claimDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ClaimDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_claimDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_claimDecl }
}
antlr4rust::tid!{ClaimDeclContextExt<'a>}

impl<'input> ClaimDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ClaimDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ClaimDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ClaimDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ClaimDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn claimField_all(&self) ->  Vec<Rc<ClaimFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn claimField(&self, i: usize) -> Option<Rc<ClaimFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ClaimDeclContextAttrs<'input> for ClaimDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn claimDecl(&mut self,)
	-> Result<Rc<ClaimDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ClaimDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 60, RULE_claimDecl);
        let mut _localctx: Rc<ClaimDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(473);
			recog.base.match_token(EAL_T__37,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(474);
			recog.qualifiedName()?;

			recog.base.set_state(475);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(479);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(476);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(481);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(485);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 10)) & !0x3f) == 0 && ((1usize << (_la - 10)) & 537919489) != 0) {
				{
				{
				/*InvokeRule claimField*/
				recog.base.set_state(482);
				recog.claimField()?;

				}
				}
				recog.base.set_state(487);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(488);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- claimField ----------------
pub type ClaimFieldContextAll<'input> = ClaimFieldContext<'input>;


pub type ClaimFieldContext<'input> = BaseParserRuleContext<'input,ClaimFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ClaimFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ClaimFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ClaimFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_claimField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_claimField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ClaimFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_claimField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ClaimFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_claimField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_claimField }
}
antlr4rust::tid!{ClaimFieldContextExt<'a>}

impl<'input> ClaimFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ClaimFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ClaimFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ClaimFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ClaimFieldContextExt<'input>>{

fn statementField(&self) -> Option<Rc<StatementFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn environmentField(&self) -> Option<Rc<EnvironmentFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn propositionDecl(&self) -> Option<Rc<PropositionDeclContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ClaimFieldContextAttrs<'input> for ClaimFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn claimField(&mut self,)
	-> Result<Rc<ClaimFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ClaimFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 62, RULE_claimField);
        let mut _localctx: Rc<ClaimFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(493);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__29 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule statementField*/
					recog.base.set_state(490);
					recog.statementField()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(491);
					recog.environmentField()?;

					}
				}

			EAL_T__38 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule propositionDecl*/
					recog.base.set_state(492);
					recog.propositionDecl()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- propositionDecl ----------------
pub type PropositionDeclContextAll<'input> = PropositionDeclContext<'input>;


pub type PropositionDeclContext<'input> = BaseParserRuleContext<'input,PropositionDeclContextExt<'input>>;

#[derive(Clone)]
pub struct PropositionDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PropositionDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PropositionDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_propositionDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_propositionDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PropositionDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_propositionDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for PropositionDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_propositionDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_propositionDecl }
}
antlr4rust::tid!{PropositionDeclContextExt<'a>}

impl<'input> PropositionDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PropositionDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PropositionDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PropositionDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PropositionDeclContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn propositionField_all(&self) ->  Vec<Rc<PropositionFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn propositionField(&self, i: usize) -> Option<Rc<PropositionFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> PropositionDeclContextAttrs<'input> for PropositionDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn propositionDecl(&mut self,)
	-> Result<Rc<PropositionDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PropositionDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 64, RULE_propositionDecl);
        let mut _localctx: Rc<PropositionDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(495);
			recog.base.match_token(EAL_T__38,&mut recog.err_handler)?;

			recog.base.set_state(496);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(500);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(497);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(502);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(506);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 18)) & !0x3f) == 0 && ((1usize << (_la - 18)) & 4194431) != 0) {
				{
				{
				/*InvokeRule propositionField*/
				recog.base.set_state(503);
				recog.propositionField()?;

				}
				}
				recog.base.set_state(508);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(509);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			recog.base.set_state(513);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(510);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(515);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- propositionField ----------------
pub type PropositionFieldContextAll<'input> = PropositionFieldContext<'input>;


pub type PropositionFieldContext<'input> = BaseParserRuleContext<'input,PropositionFieldContextExt<'input>>;

#[derive(Clone)]
pub struct PropositionFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PropositionFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PropositionFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_propositionField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_propositionField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PropositionFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_propositionField(self);
	}
}

impl<'input> CustomRuleContext<'input> for PropositionFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_propositionField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_propositionField }
}
antlr4rust::tid!{PropositionFieldContextExt<'a>}

impl<'input> PropositionFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PropositionFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PropositionFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PropositionFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PropositionFieldContextExt<'input>>{

fn subjectField(&self) -> Option<Rc<SubjectFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn quantityField(&self) -> Option<Rc<QuantityFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn unitField(&self) -> Option<Rc<UnitFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn scopeField(&self) -> Option<Rc<ScopeFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn validFromField(&self) -> Option<Rc<ValidFromFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn validUntilField(&self) -> Option<Rc<ValidUntilFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn queryField(&self) -> Option<Rc<QueryFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn resultField(&self) -> Option<Rc<ResultFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PropositionFieldContextAttrs<'input> for PropositionFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn propositionField(&mut self,)
	-> Result<Rc<PropositionFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PropositionFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 66, RULE_propositionField);
        let mut _localctx: Rc<PropositionFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(524);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__17 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule subjectField*/
					recog.base.set_state(516);
					recog.subjectField()?;

					}
				}

			EAL_T__18 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule quantityField*/
					recog.base.set_state(517);
					recog.quantityField()?;

					}
				}

			EAL_T__19 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule unitField*/
					recog.base.set_state(518);
					recog.unitField()?;

					}
				}

			EAL_T__20 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule scopeField*/
					recog.base.set_state(519);
					recog.scopeField()?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule validFromField*/
					recog.base.set_state(520);
					recog.validFromField()?;

					}
				}

			EAL_T__22 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule validUntilField*/
					recog.base.set_state(521);
					recog.validUntilField()?;

					}
				}

			EAL_T__23 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule queryField*/
					recog.base.set_state(522);
					recog.queryField()?;

					}
				}

			EAL_T__39 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule resultField*/
					recog.base.set_state(523);
					recog.resultField()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- subjectField ----------------
pub type SubjectFieldContextAll<'input> = SubjectFieldContext<'input>;


pub type SubjectFieldContext<'input> = BaseParserRuleContext<'input,SubjectFieldContextExt<'input>>;

#[derive(Clone)]
pub struct SubjectFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for SubjectFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for SubjectFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_subjectField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_subjectField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for SubjectFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_subjectField(self);
	}
}

impl<'input> CustomRuleContext<'input> for SubjectFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_subjectField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_subjectField }
}
antlr4rust::tid!{SubjectFieldContextExt<'a>}

impl<'input> SubjectFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SubjectFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SubjectFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait SubjectFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<SubjectFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> SubjectFieldContextAttrs<'input> for SubjectFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn subjectField(&mut self,)
	-> Result<Rc<SubjectFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SubjectFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 68, RULE_subjectField);
        let mut _localctx: Rc<SubjectFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(526);
			recog.base.match_token(EAL_T__17,&mut recog.err_handler)?;

			recog.base.set_state(527);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(528);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- quantityField ----------------
pub type QuantityFieldContextAll<'input> = QuantityFieldContext<'input>;


pub type QuantityFieldContext<'input> = BaseParserRuleContext<'input,QuantityFieldContextExt<'input>>;

#[derive(Clone)]
pub struct QuantityFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for QuantityFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for QuantityFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_quantityField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_quantityField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for QuantityFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_quantityField(self);
	}
}

impl<'input> CustomRuleContext<'input> for QuantityFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_quantityField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_quantityField }
}
antlr4rust::tid!{QuantityFieldContextExt<'a>}

impl<'input> QuantityFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<QuantityFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,QuantityFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait QuantityFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<QuantityFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> QuantityFieldContextAttrs<'input> for QuantityFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn quantityField(&mut self,)
	-> Result<Rc<QuantityFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = QuantityFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 70, RULE_quantityField);
        let mut _localctx: Rc<QuantityFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(530);
			recog.base.match_token(EAL_T__18,&mut recog.err_handler)?;

			recog.base.set_state(531);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(532);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- unitField ----------------
pub type UnitFieldContextAll<'input> = UnitFieldContext<'input>;


pub type UnitFieldContext<'input> = BaseParserRuleContext<'input,UnitFieldContextExt<'input>>;

#[derive(Clone)]
pub struct UnitFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for UnitFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for UnitFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_unitField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_unitField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for UnitFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_unitField(self);
	}
}

impl<'input> CustomRuleContext<'input> for UnitFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_unitField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_unitField }
}
antlr4rust::tid!{UnitFieldContextExt<'a>}

impl<'input> UnitFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<UnitFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,UnitFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait UnitFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<UnitFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> UnitFieldContextAttrs<'input> for UnitFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn unitField(&mut self,)
	-> Result<Rc<UnitFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = UnitFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 72, RULE_unitField);
        let mut _localctx: Rc<UnitFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(534);
			recog.base.match_token(EAL_T__19,&mut recog.err_handler)?;

			recog.base.set_state(535);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(536);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- scopeField ----------------
pub type ScopeFieldContextAll<'input> = ScopeFieldContext<'input>;


pub type ScopeFieldContext<'input> = BaseParserRuleContext<'input,ScopeFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ScopeFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ScopeFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ScopeFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_scopeField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_scopeField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ScopeFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_scopeField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ScopeFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_scopeField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_scopeField }
}
antlr4rust::tid!{ScopeFieldContextExt<'a>}

impl<'input> ScopeFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ScopeFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ScopeFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ScopeFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ScopeFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ScopeFieldContextAttrs<'input> for ScopeFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn scopeField(&mut self,)
	-> Result<Rc<ScopeFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ScopeFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 74, RULE_scopeField);
        let mut _localctx: Rc<ScopeFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(538);
			recog.base.match_token(EAL_T__20,&mut recog.err_handler)?;

			recog.base.set_state(539);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(540);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- queryField ----------------
pub type QueryFieldContextAll<'input> = QueryFieldContext<'input>;


pub type QueryFieldContext<'input> = BaseParserRuleContext<'input,QueryFieldContextExt<'input>>;

#[derive(Clone)]
pub struct QueryFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for QueryFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for QueryFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_queryField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_queryField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for QueryFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_queryField(self);
	}
}

impl<'input> CustomRuleContext<'input> for QueryFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_queryField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_queryField }
}
antlr4rust::tid!{QueryFieldContextExt<'a>}

impl<'input> QueryFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<QueryFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,QueryFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait QueryFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<QueryFieldContextExt<'input>>{

fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> QueryFieldContextAttrs<'input> for QueryFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn queryField(&mut self,)
	-> Result<Rc<QueryFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = QueryFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 76, RULE_queryField);
        let mut _localctx: Rc<QueryFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(542);
			recog.base.match_token(EAL_T__23,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(543);
			recog.jsonValue()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(544);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- resultField ----------------
pub type ResultFieldContextAll<'input> = ResultFieldContext<'input>;


pub type ResultFieldContext<'input> = BaseParserRuleContext<'input,ResultFieldContextExt<'input>>;

#[derive(Clone)]
pub struct ResultFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ResultFieldContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ResultFieldContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_resultField(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_resultField(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ResultFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_resultField(self);
	}
}

impl<'input> CustomRuleContext<'input> for ResultFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_resultField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_resultField }
}
antlr4rust::tid!{ResultFieldContextExt<'a>}

impl<'input> ResultFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ResultFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ResultFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ResultFieldContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ResultFieldContextExt<'input>>{

fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ResultFieldContextAttrs<'input> for ResultFieldContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn resultField(&mut self,)
	-> Result<Rc<ResultFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ResultFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 78, RULE_resultField);
        let mut _localctx: Rc<ResultFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(546);
			recog.base.match_token(EAL_T__39,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(547);
			recog.expression()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(548);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- argumentDecl ----------------
pub type ArgumentDeclContextAll<'input> = ArgumentDeclContext<'input>;


pub type ArgumentDeclContext<'input> = BaseParserRuleContext<'input,ArgumentDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ArgumentDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ArgumentDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ArgumentDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_argumentDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_argumentDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ArgumentDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_argumentDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ArgumentDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_argumentDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_argumentDecl }
}
antlr4rust::tid!{ArgumentDeclContextExt<'a>}

impl<'input> ArgumentDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ArgumentDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ArgumentDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ArgumentDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ArgumentDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn argumentFlow(&self) -> Option<Rc<ArgumentFlowContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ArgumentDeclContextAttrs<'input> for ArgumentDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn argumentDecl(&mut self,)
	-> Result<Rc<ArgumentDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ArgumentDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 80, RULE_argumentDecl);
        let mut _localctx: Rc<ArgumentDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(550);
			recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(551);
			recog.qualifiedName()?;

			recog.base.set_state(585);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__41 
				=> {
					{
					recog.base.set_state(552);
					recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

					recog.base.set_state(556);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(553);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(558);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(559);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(560);
					recog.lineEnd()?;

					}
				}

			EAL_T__3 
				=> {
					{
					recog.base.set_state(562);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

					recog.base.set_state(566);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(563);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(568);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(578);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(569);
						recog.declaration()?;

						recog.base.set_state(573);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(570);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(575);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(580);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(581);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(582);
					recog.lineEnd()?;

					recog.base.set_state(583);
					recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- argumentFlow ----------------
pub type ArgumentFlowContextAll<'input> = ArgumentFlowContext<'input>;


pub type ArgumentFlowContext<'input> = BaseParserRuleContext<'input,ArgumentFlowContextExt<'input>>;

#[derive(Clone)]
pub struct ArgumentFlowContextExt<'input>{
	pub reasoningRef: Option<Rc<ReferenceContextAll<'input>>>,
	pub conclusionRef: Option<Rc<ReferenceContextAll<'input>>>,
	pub bindingRef: Option<Rc<ReferenceContextAll<'input>>>,
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ArgumentFlowContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ArgumentFlowContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_argumentFlow(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_argumentFlow(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ArgumentFlowContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_argumentFlow(self);
	}
}

impl<'input> CustomRuleContext<'input> for ArgumentFlowContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_argumentFlow }
	//fn type_rule_index() -> usize where Self: Sized { RULE_argumentFlow }
}
antlr4rust::tid!{ArgumentFlowContextExt<'a>}

impl<'input> ArgumentFlowContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ArgumentFlowContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ArgumentFlowContextExt{
				reasoningRef: None, conclusionRef: None, bindingRef: None, 

				ph:PhantomData
			}),
		)
	}
}

pub trait ArgumentFlowContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ArgumentFlowContextExt<'input>>{

fn support(&self) -> Option<Rc<SupportContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn reference_all(&self) ->  Vec<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reference(&self, i: usize) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> ArgumentFlowContextAttrs<'input> for ArgumentFlowContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn argumentFlow(&mut self,)
	-> Result<Rc<ArgumentFlowContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ArgumentFlowContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 82, RULE_argumentFlow);
        let mut _localctx: Rc<ArgumentFlowContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule support*/
			recog.base.set_state(587);
			recog.support()?;

			recog.base.set_state(591);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(588);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(593);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(594);
			recog.base.match_token(EAL_T__42,&mut recog.err_handler)?;

			recog.base.set_state(598);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(595);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(600);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(601);
			let tmp = recog.reference()?;
			// SAFETY: the borrow only covers this one field access, during which no other handle to the context is dereferenced.
			(unsafe { cast_mut::<ArgumentFlowContext >(&mut _localctx) }).reasoningRef = Some(tmp.clone());
			  

			recog.base.set_state(605);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(602);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(607);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(608);
			recog.base.match_token(EAL_T__43,&mut recog.err_handler)?;

			recog.base.set_state(612);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(609);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(614);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(615);
			let tmp = recog.reference()?;
			// SAFETY: the borrow only covers this one field access, during which no other handle to the context is dereferenced.
			(unsafe { cast_mut::<ArgumentFlowContext >(&mut _localctx) }).conclusionRef = Some(tmp.clone());
			  

			recog.base.set_state(624);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(43,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(619);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(616);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(621);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(622);
					recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

					/*InvokeRule reference*/
					recog.base.set_state(623);
					let tmp = recog.reference()?;
					// SAFETY: the borrow only covers this one field access, during which no other handle to the context is dereferenced.
					(unsafe { cast_mut::<ArgumentFlowContext >(&mut _localctx) }).bindingRef = Some(tmp.clone());
					  

					}
				}

				_ => {}
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- support ----------------
pub type SupportContextAll<'input> = SupportContext<'input>;


pub type SupportContext<'input> = BaseParserRuleContext<'input,SupportContextExt<'input>>;

#[derive(Clone)]
pub struct SupportContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for SupportContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for SupportContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_support(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_support(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for SupportContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_support(self);
	}
}

impl<'input> CustomRuleContext<'input> for SupportContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_support }
	//fn type_rule_index() -> usize where Self: Sized { RULE_support }
}
antlr4rust::tid!{SupportContextExt<'a>}

impl<'input> SupportContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SupportContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SupportContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait SupportContextAttrs<'input>: EALParserContext<'input> + BorrowMut<SupportContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn supportGroup_all(&self) ->  Vec<Rc<SupportGroupContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn supportGroup(&self, i: usize) -> Option<Rc<SupportGroupContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> SupportContextAttrs<'input> for SupportContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn support(&mut self,)
	-> Result<Rc<SupportContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SupportContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 84, RULE_support);
        let mut _localctx: Rc<SupportContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(626);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

			recog.base.set_state(630);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(44,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(627);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(632);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(44,&mut recog.base)?;
			}
			recog.base.set_state(647);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if ((((_la - 25)) & !0x3f) == 0 && ((1usize << (_la - 25)) & 25165825) != 0) {
				{
				/*InvokeRule supportGroup*/
				recog.base.set_state(633);
				recog.supportGroup()?;

				recog.base.set_state(644);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__8 {
					{
					{
					recog.base.set_state(634);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(638);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(635);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(640);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule supportGroup*/
					recog.base.set_state(641);
					recog.supportGroup()?;

					}
					}
					recog.base.set_state(646);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(652);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(649);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(654);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(655);
			recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- supportGroup ----------------
pub type SupportGroupContextAll<'input> = SupportGroupContext<'input>;


pub type SupportGroupContext<'input> = BaseParserRuleContext<'input,SupportGroupContextExt<'input>>;

#[derive(Clone)]
pub struct SupportGroupContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for SupportGroupContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for SupportGroupContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_supportGroup(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_supportGroup(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for SupportGroupContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_supportGroup(self);
	}
}

impl<'input> CustomRuleContext<'input> for SupportGroupContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_supportGroup }
	//fn type_rule_index() -> usize where Self: Sized { RULE_supportGroup }
}
antlr4rust::tid!{SupportGroupContextExt<'a>}

impl<'input> SupportGroupContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SupportGroupContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SupportGroupContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait SupportGroupContextAttrs<'input>: EALParserContext<'input> + BorrowMut<SupportGroupContextExt<'input>>{

fn supportKind(&self) -> Option<Rc<SupportKindContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn reference_all(&self) ->  Vec<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reference(&self, i: usize) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> SupportGroupContextAttrs<'input> for SupportGroupContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn supportGroup(&mut self,)
	-> Result<Rc<SupportGroupContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SupportGroupContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 86, RULE_supportGroup);
        let mut _localctx: Rc<SupportGroupContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule supportKind*/
			recog.base.set_state(657);
			recog.supportKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(658);
			recog.reference()?;

			recog.base.set_state(669);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(50,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(659);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(663);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(660);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(665);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule reference*/
					recog.base.set_state(666);
					recog.reference()?;

					}
					} 
				}
				recog.base.set_state(671);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(50,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- supportKind ----------------
pub type SupportKindContextAll<'input> = SupportKindContext<'input>;


pub type SupportKindContext<'input> = BaseParserRuleContext<'input,SupportKindContextExt<'input>>;

#[derive(Clone)]
pub struct SupportKindContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for SupportKindContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for SupportKindContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_supportKind(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_supportKind(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for SupportKindContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_supportKind(self);
	}
}

impl<'input> CustomRuleContext<'input> for SupportKindContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_supportKind }
	//fn type_rule_index() -> usize where Self: Sized { RULE_supportKind }
}
antlr4rust::tid!{SupportKindContextExt<'a>}

impl<'input> SupportKindContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SupportKindContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SupportKindContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait SupportKindContextAttrs<'input>: EALParserContext<'input> + BorrowMut<SupportKindContextExt<'input>>{


}

impl<'input> SupportKindContextAttrs<'input> for SupportKindContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn supportKind(&mut self,)
	-> Result<Rc<SupportKindContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SupportKindContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 88, RULE_supportKind);
        let mut _localctx: Rc<SupportKindContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(672);
			_la = recog.base.input.la(1);
			if { !(((((_la - 25)) & !0x3f) == 0 && ((1usize << (_la - 25)) & 25165825) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- patternDecl ----------------
pub type PatternDeclContextAll<'input> = PatternDeclContext<'input>;


pub type PatternDeclContext<'input> = BaseParserRuleContext<'input,PatternDeclContextExt<'input>>;

#[derive(Clone)]
pub struct PatternDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PatternDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PatternDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_patternDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_patternDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PatternDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_patternDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for PatternDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_patternDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_patternDecl }
}
antlr4rust::tid!{PatternDeclContextExt<'a>}

impl<'input> PatternDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PatternDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PatternDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PatternDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PatternDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn argumentFlow(&self) -> Option<Rc<ArgumentFlowContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn patternParameter_all(&self) ->  Vec<Rc<PatternParameterContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn patternParameter(&self, i: usize) -> Option<Rc<PatternParameterContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> PatternDeclContextAttrs<'input> for PatternDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn patternDecl(&mut self,)
	-> Result<Rc<PatternDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PatternDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 90, RULE_patternDecl);
        let mut _localctx: Rc<PatternDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(674);
			recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(675);
			recog.qualifiedName()?;

			recog.base.set_state(676);
			recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

			recog.base.set_state(680);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(51,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(677);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(682);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(51,&mut recog.base)?;
			}
			recog.base.set_state(697);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 490602956) != 0) || ((((_la - 34)) & !0x3f) == 0 && ((1usize << (_la - 34)) & 4243130991) != 0) || _la==EAL_T__65 || _la==EAL_ID {
				{
				/*InvokeRule patternParameter*/
				recog.base.set_state(683);
				recog.patternParameter()?;

				recog.base.set_state(694);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__8 {
					{
					{
					recog.base.set_state(684);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(688);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(685);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(690);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule patternParameter*/
					recog.base.set_state(691);
					recog.patternParameter()?;

					}
					}
					recog.base.set_state(696);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(702);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(699);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(704);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(705);
			recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

			recog.base.set_state(714);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(57,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(709);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(706);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(711);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(712);
					recog.base.match_token(EAL_T__52,&mut recog.err_handler)?;

					/*InvokeRule identifier*/
					recog.base.set_state(713);
					recog.identifier()?;

					}
				}

				_ => {}
			}
			recog.base.set_state(719);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(716);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(721);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(752);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__41 
				=> {
					{
					recog.base.set_state(722);
					recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

					recog.base.set_state(726);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(723);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(728);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(729);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(730);
					recog.lineEnd()?;

					}
				}

			EAL_T__3 
				=> {
					{
					recog.base.set_state(732);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

					recog.base.set_state(736);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(733);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(738);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(746); 
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					loop {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(739);
						recog.declaration()?;

						recog.base.set_state(743);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(740);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(745);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(748); 
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						if !((((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0)) {break}
					}
					recog.base.set_state(750);
					recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- patternParameter ----------------
pub type PatternParameterContextAll<'input> = PatternParameterContext<'input>;


pub type PatternParameterContext<'input> = BaseParserRuleContext<'input,PatternParameterContextExt<'input>>;

#[derive(Clone)]
pub struct PatternParameterContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PatternParameterContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PatternParameterContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_patternParameter(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_patternParameter(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PatternParameterContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_patternParameter(self);
	}
}

impl<'input> CustomRuleContext<'input> for PatternParameterContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_patternParameter }
	//fn type_rule_index() -> usize where Self: Sized { RULE_patternParameter }
}
antlr4rust::tid!{PatternParameterContextExt<'a>}

impl<'input> PatternParameterContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PatternParameterContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PatternParameterContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PatternParameterContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PatternParameterContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn parameterKind(&self) -> Option<Rc<ParameterKindContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PatternParameterContextAttrs<'input> for PatternParameterContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn patternParameter(&mut self,)
	-> Result<Rc<PatternParameterContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PatternParameterContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 92, RULE_patternParameter);
        let mut _localctx: Rc<PatternParameterContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule identifier*/
			recog.base.set_state(754);
			recog.identifier()?;

			recog.base.set_state(755);
			recog.base.match_token(EAL_T__53,&mut recog.err_handler)?;

			/*InvokeRule parameterKind*/
			recog.base.set_state(756);
			recog.parameterKind()?;

			recog.base.set_state(759);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__45 {
				{
				recog.base.set_state(757);
				recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

				recog.base.set_state(758);
				recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

				}
			}

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- parameterKind ----------------
pub type ParameterKindContextAll<'input> = ParameterKindContext<'input>;


pub type ParameterKindContext<'input> = BaseParserRuleContext<'input,ParameterKindContextExt<'input>>;

#[derive(Clone)]
pub struct ParameterKindContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ParameterKindContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ParameterKindContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_parameterKind(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_parameterKind(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ParameterKindContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_parameterKind(self);
	}
}

impl<'input> CustomRuleContext<'input> for ParameterKindContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_parameterKind }
	//fn type_rule_index() -> usize where Self: Sized { RULE_parameterKind }
}
antlr4rust::tid!{ParameterKindContextExt<'a>}

impl<'input> ParameterKindContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ParameterKindContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ParameterKindContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ParameterKindContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ParameterKindContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ParameterKindContextAttrs<'input> for ParameterKindContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn parameterKind(&mut self,)
	-> Result<Rc<ParameterKindContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ParameterKindContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 94, RULE_parameterKind);
        let mut _localctx: Rc<ParameterKindContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(768);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__37 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(761);
					recog.base.match_token(EAL_T__37,&mut recog.err_handler)?;

					}
				}

			EAL_T__30 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(762);
					recog.base.match_token(EAL_T__30,&mut recog.err_handler)?;

					}
				}

			EAL_T__24 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(763);
					recog.base.match_token(EAL_T__24,&mut recog.err_handler)?;

					}
				}

			EAL_T__28 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(764);
					recog.base.match_token(EAL_T__28,&mut recog.err_handler)?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(765);
					recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

					}
				}

			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(766);
					recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

					}
				}

			EAL_T__1 |EAL_T__2 |EAL_T__5 |EAL_T__6 |EAL_T__7 |EAL_T__16 |EAL_T__17 |
			EAL_T__18 |EAL_T__19 |EAL_T__20 |EAL_T__23 |EAL_T__25 |EAL_T__26 |EAL_T__27 |
			EAL_T__33 |EAL_T__34 |EAL_T__35 |EAL_T__36 |EAL_T__38 |EAL_T__39 |EAL_T__42 |
			EAL_T__44 |EAL_T__49 |EAL_T__52 |EAL_T__54 |EAL_T__55 |EAL_T__56 |EAL_T__59 |
			EAL_T__60 |EAL_T__61 |EAL_T__62 |EAL_T__63 |EAL_T__64 |EAL_T__65 |EAL_ID 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule identifier*/
					recog.base.set_state(767);
					recog.identifier()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- applicationDecl ----------------
pub type ApplicationDeclContextAll<'input> = ApplicationDeclContext<'input>;


pub type ApplicationDeclContext<'input> = BaseParserRuleContext<'input,ApplicationDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ApplicationDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ApplicationDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ApplicationDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_applicationDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_applicationDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ApplicationDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_applicationDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ApplicationDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_applicationDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_applicationDecl }
}
antlr4rust::tid!{ApplicationDeclContextExt<'a>}

impl<'input> ApplicationDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ApplicationDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ApplicationDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ApplicationDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ApplicationDeclContextExt<'input>>{

fn qualifiedName_all(&self) ->  Vec<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn qualifiedName(&self, i: usize) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn patternBinding_all(&self) ->  Vec<Rc<PatternBindingContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn patternBinding(&self, i: usize) -> Option<Rc<PatternBindingContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ApplicationDeclContextAttrs<'input> for ApplicationDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn applicationDecl(&mut self,)
	-> Result<Rc<ApplicationDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ApplicationDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 96, RULE_applicationDecl);
        let mut _localctx: Rc<ApplicationDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(770);
			recog.base.match_token(EAL_T__54,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(771);
			recog.qualifiedName()?;

			recog.base.set_state(772);
			recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(773);
			recog.qualifiedName()?;

			recog.base.set_state(774);
			recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

			recog.base.set_state(778);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(66,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(775);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(780);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(66,&mut recog.base)?;
			}
			recog.base.set_state(795);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 490602956) != 0) || ((((_la - 34)) & !0x3f) == 0 && ((1usize << (_la - 34)) & 4243130991) != 0) || _la==EAL_T__65 || _la==EAL_ID {
				{
				/*InvokeRule patternBinding*/
				recog.base.set_state(781);
				recog.patternBinding()?;

				recog.base.set_state(792);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__8 {
					{
					{
					recog.base.set_state(782);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(786);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(783);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(788);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule patternBinding*/
					recog.base.set_state(789);
					recog.patternBinding()?;

					}
					}
					recog.base.set_state(794);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(800);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(797);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(802);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(803);
			recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(804);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- patternBinding ----------------
pub type PatternBindingContextAll<'input> = PatternBindingContext<'input>;


pub type PatternBindingContext<'input> = BaseParserRuleContext<'input,PatternBindingContextExt<'input>>;

#[derive(Clone)]
pub struct PatternBindingContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PatternBindingContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PatternBindingContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_patternBinding(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_patternBinding(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PatternBindingContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_patternBinding(self);
	}
}

impl<'input> CustomRuleContext<'input> for PatternBindingContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_patternBinding }
	//fn type_rule_index() -> usize where Self: Sized { RULE_patternBinding }
}
antlr4rust::tid!{PatternBindingContextExt<'a>}

impl<'input> PatternBindingContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PatternBindingContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PatternBindingContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PatternBindingContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PatternBindingContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn bindingValue(&self) -> Option<Rc<BindingValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PatternBindingContextAttrs<'input> for PatternBindingContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn patternBinding(&mut self,)
	-> Result<Rc<PatternBindingContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PatternBindingContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 98, RULE_patternBinding);
        let mut _localctx: Rc<PatternBindingContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule identifier*/
			recog.base.set_state(806);
			recog.identifier()?;

			recog.base.set_state(807);
			recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

			/*InvokeRule bindingValue*/
			recog.base.set_state(808);
			recog.bindingValue()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- bindingValue ----------------
pub type BindingValueContextAll<'input> = BindingValueContext<'input>;


pub type BindingValueContext<'input> = BaseParserRuleContext<'input,BindingValueContextExt<'input>>;

#[derive(Clone)]
pub struct BindingValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for BindingValueContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for BindingValueContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_bindingValue(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_bindingValue(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for BindingValueContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_bindingValue(self);
	}
}

impl<'input> CustomRuleContext<'input> for BindingValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_bindingValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_bindingValue }
}
antlr4rust::tid!{BindingValueContextExt<'a>}

impl<'input> BindingValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<BindingValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BindingValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BindingValueContextAttrs<'input>: EALParserContext<'input> + BorrowMut<BindingValueContextExt<'input>>{

fn reference_all(&self) ->  Vec<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reference(&self, i: usize) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> BindingValueContextAttrs<'input> for BindingValueContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn bindingValue(&mut self,)
	-> Result<Rc<BindingValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BindingValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 100, RULE_bindingValue);
        let mut _localctx: Rc<BindingValueContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			recog.base.set_state(841);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__1 |EAL_T__2 |EAL_T__5 |EAL_T__6 |EAL_T__7 |EAL_T__16 |EAL_T__17 |
			EAL_T__18 |EAL_T__19 |EAL_T__20 |EAL_T__23 |EAL_T__25 |EAL_T__26 |EAL_T__27 |
			EAL_T__33 |EAL_T__34 |EAL_T__35 |EAL_T__36 |EAL_T__38 |EAL_T__39 |EAL_T__42 |
			EAL_T__44 |EAL_T__49 |EAL_T__52 |EAL_T__54 |EAL_T__55 |EAL_T__56 |EAL_T__59 |
			EAL_T__60 |EAL_T__61 |EAL_T__62 |EAL_T__63 |EAL_T__64 |EAL_T__65 |EAL_ID 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule reference*/
					recog.base.set_state(810);
					recog.reference()?;

					}
				}

			EAL_T__45 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(811);
					recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

					recog.base.set_state(815);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(71,&mut recog.base)?;
					while { _alt!=2 && _alt!=INVALID_ALT } {
						if _alt==1 {
							{
							{
							recog.base.set_state(812);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							} 
						}
						recog.base.set_state(817);
						recog.err_handler.sync(&mut recog.base)?;
						_alt = recog.interpreter.adaptive_predict(71,&mut recog.base)?;
					}
					recog.base.set_state(832);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if (((_la) & !0x3f) == 0 && ((1usize << _la) & 490602956) != 0) || ((((_la - 34)) & !0x3f) == 0 && ((1usize << (_la - 34)) & 4243130991) != 0) || _la==EAL_T__65 || _la==EAL_ID {
						{
						/*InvokeRule reference*/
						recog.base.set_state(818);
						recog.reference()?;

						recog.base.set_state(829);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_T__8 {
							{
							{
							recog.base.set_state(819);
							recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

							recog.base.set_state(823);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
							while _la==EAL_NL {
								{
								{
								recog.base.set_state(820);
								recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

								}
								}
								recog.base.set_state(825);
								recog.err_handler.sync(&mut recog.base)?;
								_la = recog.base.input.la(1);
							}
							/*InvokeRule reference*/
							recog.base.set_state(826);
							recog.reference()?;

							}
							}
							recog.base.set_state(831);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
					}

					recog.base.set_state(837);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(834);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(839);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(840);
					recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- patternGuard ----------------
pub type PatternGuardContextAll<'input> = PatternGuardContext<'input>;


pub type PatternGuardContext<'input> = BaseParserRuleContext<'input,PatternGuardContextExt<'input>>;

#[derive(Clone)]
pub struct PatternGuardContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PatternGuardContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PatternGuardContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_patternGuard(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_patternGuard(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PatternGuardContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_patternGuard(self);
	}
}

impl<'input> CustomRuleContext<'input> for PatternGuardContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_patternGuard }
	//fn type_rule_index() -> usize where Self: Sized { RULE_patternGuard }
}
antlr4rust::tid!{PatternGuardContextExt<'a>}

impl<'input> PatternGuardContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PatternGuardContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PatternGuardContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PatternGuardContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PatternGuardContextExt<'input>>{

fn identifier(&self) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn declaration_all(&self) ->  Vec<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn declaration(&self, i: usize) -> Option<Rc<DeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> PatternGuardContextAttrs<'input> for PatternGuardContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn patternGuard(&mut self,)
	-> Result<Rc<PatternGuardContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PatternGuardContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 102, RULE_patternGuard);
        let mut _localctx: Rc<PatternGuardContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(843);
			recog.base.match_token(EAL_T__55,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(844);
			recog.identifier()?;

			recog.base.set_state(845);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(849);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(846);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(851);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(861);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(852);
				recog.declaration()?;

				recog.base.set_state(856);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(853);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(858);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(863);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(864);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			recog.base.set_state(892);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(84,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(868);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(865);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(870);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(871);
					recog.base.match_token(EAL_T__56,&mut recog.err_handler)?;

					recog.base.set_state(872);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

					recog.base.set_state(876);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(873);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(878);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(888);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while (((_la) & !0x3f) == 0 && ((1usize << _la) & 2717912392) != 0) || ((((_la - 38)) & !0x3f) == 0 && ((1usize << (_la - 38)) & 64360457) != 0) {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(879);
						recog.declaration()?;

						recog.base.set_state(883);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(880);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(885);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(890);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(891);
					recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

					}
				}

				_ => {}
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- objectionDecl ----------------
pub type ObjectionDeclContextAll<'input> = ObjectionDeclContext<'input>;


pub type ObjectionDeclContext<'input> = BaseParserRuleContext<'input,ObjectionDeclContextExt<'input>>;

#[derive(Clone)]
pub struct ObjectionDeclContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ObjectionDeclContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ObjectionDeclContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_objectionDecl(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_objectionDecl(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ObjectionDeclContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_objectionDecl(self);
	}
}

impl<'input> CustomRuleContext<'input> for ObjectionDeclContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_objectionDecl }
	//fn type_rule_index() -> usize where Self: Sized { RULE_objectionDecl }
}
antlr4rust::tid!{ObjectionDeclContextExt<'a>}

impl<'input> ObjectionDeclContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ObjectionDeclContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ObjectionDeclContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ObjectionDeclContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ObjectionDeclContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn objectionSupport(&self) -> Option<Rc<ObjectionSupportContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn targetKind(&self) -> Option<Rc<TargetKindContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn reference(&self) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> ObjectionDeclContextAttrs<'input> for ObjectionDeclContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn objectionDecl(&mut self,)
	-> Result<Rc<ObjectionDeclContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ObjectionDeclContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 104, RULE_objectionDecl);
        let mut _localctx: Rc<ObjectionDeclContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(894);
			recog.base.match_token(EAL_T__57,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(895);
			recog.qualifiedName()?;

			recog.base.set_state(896);
			recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

			recog.base.set_state(900);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(897);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(902);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule objectionSupport*/
			recog.base.set_state(903);
			recog.objectionSupport()?;

			recog.base.set_state(907);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(904);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(909);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(910);
			recog.base.match_token(EAL_T__58,&mut recog.err_handler)?;

			recog.base.set_state(914);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(911);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(916);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule targetKind*/
			recog.base.set_state(917);
			recog.targetKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(918);
			recog.reference()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(919);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- objectionSupport ----------------
pub type ObjectionSupportContextAll<'input> = ObjectionSupportContext<'input>;


pub type ObjectionSupportContext<'input> = BaseParserRuleContext<'input,ObjectionSupportContextExt<'input>>;

#[derive(Clone)]
pub struct ObjectionSupportContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ObjectionSupportContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ObjectionSupportContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_objectionSupport(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_objectionSupport(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ObjectionSupportContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_objectionSupport(self);
	}
}

impl<'input> CustomRuleContext<'input> for ObjectionSupportContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_objectionSupport }
	//fn type_rule_index() -> usize where Self: Sized { RULE_objectionSupport }
}
antlr4rust::tid!{ObjectionSupportContextExt<'a>}

impl<'input> ObjectionSupportContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ObjectionSupportContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ObjectionSupportContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ObjectionSupportContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ObjectionSupportContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn objectionGroup_all(&self) ->  Vec<Rc<ObjectionGroupContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn objectionGroup(&self, i: usize) -> Option<Rc<ObjectionGroupContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> ObjectionSupportContextAttrs<'input> for ObjectionSupportContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn objectionSupport(&mut self,)
	-> Result<Rc<ObjectionSupportContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ObjectionSupportContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 106, RULE_objectionSupport);
        let mut _localctx: Rc<ObjectionSupportContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(921);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

			recog.base.set_state(925);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(88,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(922);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(927);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(88,&mut recog.base)?;
			}
			recog.base.set_state(942);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__24 || _la==EAL_T__48 {
				{
				/*InvokeRule objectionGroup*/
				recog.base.set_state(928);
				recog.objectionGroup()?;

				recog.base.set_state(939);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__8 {
					{
					{
					recog.base.set_state(929);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(933);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(930);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(935);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule objectionGroup*/
					recog.base.set_state(936);
					recog.objectionGroup()?;

					}
					}
					recog.base.set_state(941);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(947);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(944);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(949);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(950);
			recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- objectionGroup ----------------
pub type ObjectionGroupContextAll<'input> = ObjectionGroupContext<'input>;


pub type ObjectionGroupContext<'input> = BaseParserRuleContext<'input,ObjectionGroupContextExt<'input>>;

#[derive(Clone)]
pub struct ObjectionGroupContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ObjectionGroupContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ObjectionGroupContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_objectionGroup(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_objectionGroup(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ObjectionGroupContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_objectionGroup(self);
	}
}

impl<'input> CustomRuleContext<'input> for ObjectionGroupContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_objectionGroup }
	//fn type_rule_index() -> usize where Self: Sized { RULE_objectionGroup }
}
antlr4rust::tid!{ObjectionGroupContextExt<'a>}

impl<'input> ObjectionGroupContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ObjectionGroupContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ObjectionGroupContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ObjectionGroupContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ObjectionGroupContextExt<'input>>{

fn objectionSupportKind(&self) -> Option<Rc<ObjectionSupportKindContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn reference_all(&self) ->  Vec<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reference(&self, i: usize) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> ObjectionGroupContextAttrs<'input> for ObjectionGroupContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn objectionGroup(&mut self,)
	-> Result<Rc<ObjectionGroupContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ObjectionGroupContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 108, RULE_objectionGroup);
        let mut _localctx: Rc<ObjectionGroupContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule objectionSupportKind*/
			recog.base.set_state(952);
			recog.objectionSupportKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(953);
			recog.reference()?;

			recog.base.set_state(964);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(94,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(954);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					recog.base.set_state(958);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(955);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(960);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule reference*/
					recog.base.set_state(961);
					recog.reference()?;

					}
					} 
				}
				recog.base.set_state(966);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(94,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- objectionSupportKind ----------------
pub type ObjectionSupportKindContextAll<'input> = ObjectionSupportKindContext<'input>;


pub type ObjectionSupportKindContext<'input> = BaseParserRuleContext<'input,ObjectionSupportKindContextExt<'input>>;

#[derive(Clone)]
pub struct ObjectionSupportKindContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ObjectionSupportKindContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ObjectionSupportKindContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_objectionSupportKind(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_objectionSupportKind(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ObjectionSupportKindContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_objectionSupportKind(self);
	}
}

impl<'input> CustomRuleContext<'input> for ObjectionSupportKindContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_objectionSupportKind }
	//fn type_rule_index() -> usize where Self: Sized { RULE_objectionSupportKind }
}
antlr4rust::tid!{ObjectionSupportKindContextExt<'a>}

impl<'input> ObjectionSupportKindContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ObjectionSupportKindContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ObjectionSupportKindContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ObjectionSupportKindContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ObjectionSupportKindContextExt<'input>>{


}

impl<'input> ObjectionSupportKindContextAttrs<'input> for ObjectionSupportKindContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn objectionSupportKind(&mut self,)
	-> Result<Rc<ObjectionSupportKindContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ObjectionSupportKindContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 110, RULE_objectionSupportKind);
        let mut _localctx: Rc<ObjectionSupportKindContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(967);
			_la = recog.base.input.la(1);
			if { !(_la==EAL_T__24 || _la==EAL_T__48) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- targetKind ----------------
pub type TargetKindContextAll<'input> = TargetKindContext<'input>;


pub type TargetKindContext<'input> = BaseParserRuleContext<'input,TargetKindContextExt<'input>>;

#[derive(Clone)]
pub struct TargetKindContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for TargetKindContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for TargetKindContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_targetKind(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_targetKind(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for TargetKindContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_targetKind(self);
	}
}

impl<'input> CustomRuleContext<'input> for TargetKindContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_targetKind }
	//fn type_rule_index() -> usize where Self: Sized { RULE_targetKind }
}
antlr4rust::tid!{TargetKindContextExt<'a>}

impl<'input> TargetKindContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<TargetKindContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,TargetKindContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait TargetKindContextAttrs<'input>: EALParserContext<'input> + BorrowMut<TargetKindContextExt<'input>>{


}

impl<'input> TargetKindContextAttrs<'input> for TargetKindContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn targetKind(&mut self,)
	-> Result<Rc<TargetKindContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = TargetKindContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 112, RULE_targetKind);
        let mut _localctx: Rc<TargetKindContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(969);
			_la = recog.base.input.la(1);
			if { !(((((_la - 29)) & !0x3f) == 0 && ((1usize << (_la - 29)) & 536875525) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- argumentationDirective ----------------
pub type ArgumentationDirectiveContextAll<'input> = ArgumentationDirectiveContext<'input>;


pub type ArgumentationDirectiveContext<'input> = BaseParserRuleContext<'input,ArgumentationDirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct ArgumentationDirectiveContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ArgumentationDirectiveContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ArgumentationDirectiveContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_argumentationDirective(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_argumentationDirective(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ArgumentationDirectiveContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_argumentationDirective(self);
	}
}

impl<'input> CustomRuleContext<'input> for ArgumentationDirectiveContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_argumentationDirective }
	//fn type_rule_index() -> usize where Self: Sized { RULE_argumentationDirective }
}
antlr4rust::tid!{ArgumentationDirectiveContextExt<'a>}

impl<'input> ArgumentationDirectiveContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ArgumentationDirectiveContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ArgumentationDirectiveContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ArgumentationDirectiveContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ArgumentationDirectiveContextExt<'input>>{

fn qualifiedName_all(&self) ->  Vec<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn qualifiedName(&self, i: usize) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn signedNumber(&self) -> Option<Rc<SignedNumberContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ArgumentationDirectiveContextAttrs<'input> for ArgumentationDirectiveContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn argumentationDirective(&mut self,)
	-> Result<Rc<ArgumentationDirectiveContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ArgumentationDirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 114, RULE_argumentationDirective);
        let mut _localctx: Rc<ArgumentationDirectiveContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1000);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__59 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(971);
					recog.base.match_token(EAL_T__59,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(972);
					recog.qualifiedName()?;

					recog.base.set_state(973);
					recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

					recog.base.set_state(974);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(975);
					recog.lineEnd()?;

					}
				}

			EAL_T__60 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(977);
					recog.base.match_token(EAL_T__60,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(978);
					recog.qualifiedName()?;

					/*InvokeRule signedNumber*/
					recog.base.set_state(979);
					recog.signedNumber()?;

					recog.base.set_state(980);
					recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

					recog.base.set_state(981);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(982);
					recog.lineEnd()?;

					}
				}

			EAL_T__61 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(984);
					recog.base.match_token(EAL_T__61,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(985);
					recog.qualifiedName()?;

					recog.base.set_state(986);
					recog.base.match_token(EAL_T__34,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(987);
					recog.qualifiedName()?;

					recog.base.set_state(988);
					recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

					recog.base.set_state(989);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(990);
					recog.lineEnd()?;

					}
				}

			EAL_T__62 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(992);
					recog.base.match_token(EAL_T__62,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(993);
					recog.qualifiedName()?;

					recog.base.set_state(994);
					recog.base.match_token(EAL_T__63,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(995);
					recog.qualifiedName()?;

					recog.base.set_state(996);
					recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

					recog.base.set_state(997);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(998);
					recog.lineEnd()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- referenceList ----------------
pub type ReferenceListContextAll<'input> = ReferenceListContext<'input>;


pub type ReferenceListContext<'input> = BaseParserRuleContext<'input,ReferenceListContextExt<'input>>;

#[derive(Clone)]
pub struct ReferenceListContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ReferenceListContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ReferenceListContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_referenceList(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_referenceList(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ReferenceListContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_referenceList(self);
	}
}

impl<'input> CustomRuleContext<'input> for ReferenceListContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_referenceList }
	//fn type_rule_index() -> usize where Self: Sized { RULE_referenceList }
}
antlr4rust::tid!{ReferenceListContextExt<'a>}

impl<'input> ReferenceListContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ReferenceListContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ReferenceListContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ReferenceListContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ReferenceListContextExt<'input>>{

fn reference_all(&self) ->  Vec<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn reference(&self, i: usize) -> Option<Rc<ReferenceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}

}

impl<'input> ReferenceListContextAttrs<'input> for ReferenceListContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn referenceList(&mut self,)
	-> Result<Rc<ReferenceListContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ReferenceListContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 116, RULE_referenceList);
        let mut _localctx: Rc<ReferenceListContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1002);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

			recog.base.set_state(1006);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1003);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1008);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(1009);
			recog.reference()?;

			recog.base.set_state(1020);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__8 {
				{
				{
				recog.base.set_state(1010);
				recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

				recog.base.set_state(1014);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1011);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1016);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				/*InvokeRule reference*/
				recog.base.set_state(1017);
				recog.reference()?;

				}
				}
				recog.base.set_state(1022);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1026);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1023);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1028);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1029);
			recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- reference ----------------
pub type ReferenceContextAll<'input> = ReferenceContext<'input>;


pub type ReferenceContext<'input> = BaseParserRuleContext<'input,ReferenceContextExt<'input>>;

#[derive(Clone)]
pub struct ReferenceContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ReferenceContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ReferenceContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_reference(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_reference(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ReferenceContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_reference(self);
	}
}

impl<'input> CustomRuleContext<'input> for ReferenceContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_reference }
	//fn type_rule_index() -> usize where Self: Sized { RULE_reference }
}
antlr4rust::tid!{ReferenceContextExt<'a>}

impl<'input> ReferenceContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ReferenceContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ReferenceContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ReferenceContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ReferenceContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ReferenceContextAttrs<'input> for ReferenceContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn reference(&mut self,)
	-> Result<Rc<ReferenceContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ReferenceContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 118, RULE_reference);
        let mut _localctx: Rc<ReferenceContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1042);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(100,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1031);
					recog.qualifiedName()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(1032);
					recog.base.match_token(EAL_T__64,&mut recog.err_handler)?;

					recog.base.set_state(1033);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(1034);
					recog.qualifiedName()?;

					recog.base.set_state(1035);
					recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(1037);
					recog.base.match_token(EAL_T__65,&mut recog.err_handler)?;

					recog.base.set_state(1038);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(1039);
					recog.qualifiedName()?;

					recog.base.set_state(1040);
					recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- qualifiedName ----------------
pub type QualifiedNameContextAll<'input> = QualifiedNameContext<'input>;


pub type QualifiedNameContext<'input> = BaseParserRuleContext<'input,QualifiedNameContextExt<'input>>;

#[derive(Clone)]
pub struct QualifiedNameContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for QualifiedNameContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for QualifiedNameContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_qualifiedName(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_qualifiedName(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for QualifiedNameContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_qualifiedName(self);
	}
}

impl<'input> CustomRuleContext<'input> for QualifiedNameContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_qualifiedName }
	//fn type_rule_index() -> usize where Self: Sized { RULE_qualifiedName }
}
antlr4rust::tid!{QualifiedNameContextExt<'a>}

impl<'input> QualifiedNameContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<QualifiedNameContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,QualifiedNameContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait QualifiedNameContextAttrs<'input>: EALParserContext<'input> + BorrowMut<QualifiedNameContextExt<'input>>{

fn identifier_all(&self) ->  Vec<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn identifier(&self, i: usize) -> Option<Rc<IdentifierContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> QualifiedNameContextAttrs<'input> for QualifiedNameContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn qualifiedName(&mut self,)
	-> Result<Rc<QualifiedNameContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = QualifiedNameContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 120, RULE_qualifiedName);
        let mut _localctx: Rc<QualifiedNameContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule identifier*/
			recog.base.set_state(1044);
			recog.identifier()?;

			recog.base.set_state(1049);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__66 {
				{
				{
				recog.base.set_state(1045);
				recog.base.match_token(EAL_T__66,&mut recog.err_handler)?;

				/*InvokeRule identifier*/
				recog.base.set_state(1046);
				recog.identifier()?;

				}
				}
				recog.base.set_state(1051);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- predicate ----------------
pub type PredicateContextAll<'input> = PredicateContext<'input>;


pub type PredicateContext<'input> = BaseParserRuleContext<'input,PredicateContextExt<'input>>;

#[derive(Clone)]
pub struct PredicateContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PredicateContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PredicateContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_predicate(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_predicate(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PredicateContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_predicate(self);
	}
}

impl<'input> CustomRuleContext<'input> for PredicateContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_predicate }
	//fn type_rule_index() -> usize where Self: Sized { RULE_predicate }
}
antlr4rust::tid!{PredicateContextExt<'a>}

impl<'input> PredicateContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PredicateContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PredicateContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PredicateContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PredicateContextExt<'input>>{

fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PredicateContextAttrs<'input> for PredicateContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn predicate(&mut self,)
	-> Result<Rc<PredicateContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PredicateContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 122, RULE_predicate);
        let mut _localctx: Rc<PredicateContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1052);
			recog.base.match_token(EAL_T__67,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(1053);
			recog.expression()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(1054);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- expression ----------------
pub type ExpressionContextAll<'input> = ExpressionContext<'input>;


pub type ExpressionContext<'input> = BaseParserRuleContext<'input,ExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_expression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_expression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_expression(self);
	}
}

impl<'input> CustomRuleContext<'input> for ExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_expression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_expression }
}
antlr4rust::tid!{ExpressionContextExt<'a>}

impl<'input> ExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ExpressionContextExt<'input>>{

fn orExpression(&self) -> Option<Rc<OrExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ExpressionContextAttrs<'input> for ExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn expression(&mut self,)
	-> Result<Rc<ExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 124, RULE_expression);
        let mut _localctx: Rc<ExpressionContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule orExpression*/
			recog.base.set_state(1056);
			recog.orExpression()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- orExpression ----------------
pub type OrExpressionContextAll<'input> = OrExpressionContext<'input>;


pub type OrExpressionContext<'input> = BaseParserRuleContext<'input,OrExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct OrExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for OrExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for OrExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_orExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_orExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for OrExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_orExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for OrExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_orExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_orExpression }
}
antlr4rust::tid!{OrExpressionContextExt<'a>}

impl<'input> OrExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<OrExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,OrExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait OrExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<OrExpressionContextExt<'input>>{

fn andExpression_all(&self) ->  Vec<Rc<AndExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn andExpression(&self, i: usize) -> Option<Rc<AndExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> OrExpressionContextAttrs<'input> for OrExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn orExpression(&mut self,)
	-> Result<Rc<OrExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = OrExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 126, RULE_orExpression);
        let mut _localctx: Rc<OrExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule andExpression*/
			recog.base.set_state(1058);
			recog.andExpression()?;

			recog.base.set_state(1063);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__68 {
				{
				{
				recog.base.set_state(1059);
				recog.base.match_token(EAL_T__68,&mut recog.err_handler)?;

				/*InvokeRule andExpression*/
				recog.base.set_state(1060);
				recog.andExpression()?;

				}
				}
				recog.base.set_state(1065);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- andExpression ----------------
pub type AndExpressionContextAll<'input> = AndExpressionContext<'input>;


pub type AndExpressionContext<'input> = BaseParserRuleContext<'input,AndExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct AndExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for AndExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for AndExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_andExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_andExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for AndExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_andExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for AndExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_andExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_andExpression }
}
antlr4rust::tid!{AndExpressionContextExt<'a>}

impl<'input> AndExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<AndExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AndExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AndExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<AndExpressionContextExt<'input>>{

fn comparisonExpression_all(&self) ->  Vec<Rc<ComparisonExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn comparisonExpression(&self, i: usize) -> Option<Rc<ComparisonExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> AndExpressionContextAttrs<'input> for AndExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn andExpression(&mut self,)
	-> Result<Rc<AndExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AndExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 128, RULE_andExpression);
        let mut _localctx: Rc<AndExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule comparisonExpression*/
			recog.base.set_state(1066);
			recog.comparisonExpression()?;

			recog.base.set_state(1071);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__69 {
				{
				{
				recog.base.set_state(1067);
				recog.base.match_token(EAL_T__69,&mut recog.err_handler)?;

				/*InvokeRule comparisonExpression*/
				recog.base.set_state(1068);
				recog.comparisonExpression()?;

				}
				}
				recog.base.set_state(1073);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- comparisonExpression ----------------
pub type ComparisonExpressionContextAll<'input> = ComparisonExpressionContext<'input>;


pub type ComparisonExpressionContext<'input> = BaseParserRuleContext<'input,ComparisonExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ComparisonExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ComparisonExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ComparisonExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_comparisonExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_comparisonExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ComparisonExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_comparisonExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for ComparisonExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_comparisonExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_comparisonExpression }
}
antlr4rust::tid!{ComparisonExpressionContextExt<'a>}

impl<'input> ComparisonExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ComparisonExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ComparisonExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ComparisonExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ComparisonExpressionContextExt<'input>>{

fn additiveExpression_all(&self) ->  Vec<Rc<AdditiveExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn additiveExpression(&self, i: usize) -> Option<Rc<AdditiveExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn comparator(&self) -> Option<Rc<ComparatorContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> ComparisonExpressionContextAttrs<'input> for ComparisonExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn comparisonExpression(&mut self,)
	-> Result<Rc<ComparisonExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ComparisonExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 130, RULE_comparisonExpression);
        let mut _localctx: Rc<ComparisonExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule additiveExpression*/
			recog.base.set_state(1074);
			recog.additiveExpression()?;

			recog.base.set_state(1078);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if ((((_la - 77)) & !0x3f) == 0 && ((1usize << (_la - 77)) & 63) != 0) {
				{
				/*InvokeRule comparator*/
				recog.base.set_state(1075);
				recog.comparator()?;

				/*InvokeRule additiveExpression*/
				recog.base.set_state(1076);
				recog.additiveExpression()?;

				}
			}

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- additiveExpression ----------------
pub type AdditiveExpressionContextAll<'input> = AdditiveExpressionContext<'input>;


pub type AdditiveExpressionContext<'input> = BaseParserRuleContext<'input,AdditiveExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct AdditiveExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for AdditiveExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for AdditiveExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_additiveExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_additiveExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for AdditiveExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_additiveExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for AdditiveExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_additiveExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_additiveExpression }
}
antlr4rust::tid!{AdditiveExpressionContextExt<'a>}

impl<'input> AdditiveExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<AdditiveExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AdditiveExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AdditiveExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<AdditiveExpressionContextExt<'input>>{

fn multiplicativeExpression_all(&self) ->  Vec<Rc<MultiplicativeExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn multiplicativeExpression(&self, i: usize) -> Option<Rc<MultiplicativeExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> AdditiveExpressionContextAttrs<'input> for AdditiveExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn additiveExpression(&mut self,)
	-> Result<Rc<AdditiveExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AdditiveExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 132, RULE_additiveExpression);
        let mut _localctx: Rc<AdditiveExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule multiplicativeExpression*/
			recog.base.set_state(1080);
			recog.multiplicativeExpression()?;

			recog.base.set_state(1085);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__70 || _la==EAL_T__71 {
				{
				{
				recog.base.set_state(1081);
				_la = recog.base.input.la(1);
				if { !(_la==EAL_T__70 || _la==EAL_T__71) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				/*InvokeRule multiplicativeExpression*/
				recog.base.set_state(1082);
				recog.multiplicativeExpression()?;

				}
				}
				recog.base.set_state(1087);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- multiplicativeExpression ----------------
pub type MultiplicativeExpressionContextAll<'input> = MultiplicativeExpressionContext<'input>;


pub type MultiplicativeExpressionContext<'input> = BaseParserRuleContext<'input,MultiplicativeExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct MultiplicativeExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for MultiplicativeExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for MultiplicativeExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_multiplicativeExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_multiplicativeExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for MultiplicativeExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_multiplicativeExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for MultiplicativeExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_multiplicativeExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_multiplicativeExpression }
}
antlr4rust::tid!{MultiplicativeExpressionContextExt<'a>}

impl<'input> MultiplicativeExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<MultiplicativeExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MultiplicativeExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait MultiplicativeExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<MultiplicativeExpressionContextExt<'input>>{

fn unaryExpression_all(&self) ->  Vec<Rc<UnaryExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn unaryExpression(&self, i: usize) -> Option<Rc<UnaryExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> MultiplicativeExpressionContextAttrs<'input> for MultiplicativeExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn multiplicativeExpression(&mut self,)
	-> Result<Rc<MultiplicativeExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MultiplicativeExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 134, RULE_multiplicativeExpression);
        let mut _localctx: Rc<MultiplicativeExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule unaryExpression*/
			recog.base.set_state(1088);
			recog.unaryExpression()?;

			recog.base.set_state(1093);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 73)) & !0x3f) == 0 && ((1usize << (_la - 73)) & 7) != 0) {
				{
				{
				recog.base.set_state(1089);
				_la = recog.base.input.la(1);
				if { !(((((_la - 73)) & !0x3f) == 0 && ((1usize << (_la - 73)) & 7) != 0)) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				/*InvokeRule unaryExpression*/
				recog.base.set_state(1090);
				recog.unaryExpression()?;

				}
				}
				recog.base.set_state(1095);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- unaryExpression ----------------
pub type UnaryExpressionContextAll<'input> = UnaryExpressionContext<'input>;


pub type UnaryExpressionContext<'input> = BaseParserRuleContext<'input,UnaryExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct UnaryExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for UnaryExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for UnaryExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_unaryExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_unaryExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for UnaryExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_unaryExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for UnaryExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_unaryExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_unaryExpression }
}
antlr4rust::tid!{UnaryExpressionContextExt<'a>}

impl<'input> UnaryExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<UnaryExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,UnaryExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait UnaryExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<UnaryExpressionContextExt<'input>>{

fn unaryExpression(&self) -> Option<Rc<UnaryExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn primaryExpression(&self) -> Option<Rc<PrimaryExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> UnaryExpressionContextAttrs<'input> for UnaryExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn unaryExpression(&mut self,)
	-> Result<Rc<UnaryExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = UnaryExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 136, RULE_unaryExpression);
        let mut _localctx: Rc<UnaryExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1099);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(107,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1096);
					_la = recog.base.input.la(1);
					if { !(((((_la - 71)) & !0x3f) == 0 && ((1usize << (_la - 71)) & 35) != 0)) } {
						recog.err_handler.recover_inline(&mut recog.base)?;

					}
					else {
						if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
						recog.err_handler.report_match(&mut recog.base);
						recog.base.consume(&mut recog.err_handler);
					}
					/*InvokeRule unaryExpression*/
					recog.base.set_state(1097);
					recog.unaryExpression()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule primaryExpression*/
					recog.base.set_state(1098);
					recog.primaryExpression()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- primaryExpression ----------------
pub type PrimaryExpressionContextAll<'input> = PrimaryExpressionContext<'input>;


pub type PrimaryExpressionContext<'input> = BaseParserRuleContext<'input,PrimaryExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct PrimaryExpressionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for PrimaryExpressionContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for PrimaryExpressionContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_primaryExpression(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_primaryExpression(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for PrimaryExpressionContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_primaryExpression(self);
	}
}

impl<'input> CustomRuleContext<'input> for PrimaryExpressionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_primaryExpression }
	//fn type_rule_index() -> usize where Self: Sized { RULE_primaryExpression }
}
antlr4rust::tid!{PrimaryExpressionContextExt<'a>}

impl<'input> PrimaryExpressionContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<PrimaryExpressionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PrimaryExpressionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PrimaryExpressionContextAttrs<'input>: EALParserContext<'input> + BorrowMut<PrimaryExpressionContextExt<'input>>{

fn qualifiedName(&self) -> Option<Rc<QualifiedNameContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn expression_all(&self) ->  Vec<Rc<ExpressionContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn expression(&self, i: usize) -> Option<Rc<ExpressionContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn jsonValue(&self) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PrimaryExpressionContextAttrs<'input> for PrimaryExpressionContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn primaryExpression(&mut self,)
	-> Result<Rc<PrimaryExpressionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PrimaryExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 138, RULE_primaryExpression);
        let mut _localctx: Rc<PrimaryExpressionContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1121);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(110,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1101);
					recog.qualifiedName()?;

					recog.base.set_state(1102);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					recog.base.set_state(1111);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if (((_la) & !0x3f) == 0 && ((1usize << _la) & 490602972) != 0) || ((((_la - 34)) & !0x3f) == 0 && ((1usize << (_la - 34)) & 4243266159) != 0) || ((((_la - 66)) & !0x3f) == 0 && ((1usize << (_la - 66)) & 8258657) != 0) {
						{
						/*InvokeRule expression*/
						recog.base.set_state(1103);
						recog.expression()?;

						recog.base.set_state(1108);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_T__8 {
							{
							{
							recog.base.set_state(1104);
							recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

							/*InvokeRule expression*/
							recog.base.set_state(1105);
							recog.expression()?;

							}
							}
							recog.base.set_state(1110);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
					}

					recog.base.set_state(1113);
					recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1115);
					recog.qualifiedName()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule jsonValue*/
					recog.base.set_state(1116);
					recog.jsonValue()?;

					}
				}
			,
				4 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(1117);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					/*InvokeRule expression*/
					recog.base.set_state(1118);
					recog.expression()?;

					recog.base.set_state(1119);
					recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- comparator ----------------
pub type ComparatorContextAll<'input> = ComparatorContext<'input>;


pub type ComparatorContext<'input> = BaseParserRuleContext<'input,ComparatorContextExt<'input>>;

#[derive(Clone)]
pub struct ComparatorContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for ComparatorContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for ComparatorContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_comparator(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_comparator(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for ComparatorContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_comparator(self);
	}
}

impl<'input> CustomRuleContext<'input> for ComparatorContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_comparator }
	//fn type_rule_index() -> usize where Self: Sized { RULE_comparator }
}
antlr4rust::tid!{ComparatorContextExt<'a>}

impl<'input> ComparatorContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<ComparatorContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,ComparatorContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait ComparatorContextAttrs<'input>: EALParserContext<'input> + BorrowMut<ComparatorContextExt<'input>>{


}

impl<'input> ComparatorContextAttrs<'input> for ComparatorContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn comparator(&mut self,)
	-> Result<Rc<ComparatorContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = ComparatorContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 140, RULE_comparator);
        let mut _localctx: Rc<ComparatorContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1123);
			_la = recog.base.input.la(1);
			if { !(((((_la - 77)) & !0x3f) == 0 && ((1usize << (_la - 77)) & 63) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonValue ----------------
pub type JsonValueContextAll<'input> = JsonValueContext<'input>;


pub type JsonValueContext<'input> = BaseParserRuleContext<'input,JsonValueContextExt<'input>>;

#[derive(Clone)]
pub struct JsonValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for JsonValueContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for JsonValueContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonValue(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonValue(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for JsonValueContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_jsonValue(self);
	}
}

impl<'input> CustomRuleContext<'input> for JsonValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonValue }
}
antlr4rust::tid!{JsonValueContextExt<'a>}

impl<'input> JsonValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonValueContextAttrs<'input>: EALParserContext<'input> + BorrowMut<JsonValueContextExt<'input>>{

fn jsonScalar(&self) -> Option<Rc<JsonScalarContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonObject(&self) -> Option<Rc<JsonObjectContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn jsonArray(&self) -> Option<Rc<JsonArrayContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> JsonValueContextAttrs<'input> for JsonValueContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonValue(&mut self,)
	-> Result<Rc<JsonValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 142, RULE_jsonValue);
        let mut _localctx: Rc<JsonValueContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1128);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__71 |EAL_T__82 |EAL_T__83 |EAL_T__84 |EAL_NUMBER |EAL_STRING 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule jsonScalar*/
					recog.base.set_state(1125);
					recog.jsonScalar()?;

					}
				}

			EAL_T__3 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule jsonObject*/
					recog.base.set_state(1126);
					recog.jsonObject()?;

					}
				}

			EAL_T__45 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule jsonArray*/
					recog.base.set_state(1127);
					recog.jsonArray()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonObject ----------------
pub type JsonObjectContextAll<'input> = JsonObjectContext<'input>;


pub type JsonObjectContext<'input> = BaseParserRuleContext<'input,JsonObjectContextExt<'input>>;

#[derive(Clone)]
pub struct JsonObjectContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for JsonObjectContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for JsonObjectContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonObject(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonObject(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for JsonObjectContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_jsonObject(self);
	}
}

impl<'input> CustomRuleContext<'input> for JsonObjectContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonObject }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonObject }
}
antlr4rust::tid!{JsonObjectContextExt<'a>}

impl<'input> JsonObjectContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonObjectContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonObjectContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonObjectContextAttrs<'input>: EALParserContext<'input> + BorrowMut<JsonObjectContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
/// Retrieves all `TerminalNode`s corresponding to token STRING in current rule
fn STRING_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token STRING, starting from 0.
/// Returns `None` if number of children corresponding to token STRING is less or equal than `i`.
fn STRING(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, i)
}
fn jsonValue_all(&self) ->  Vec<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn jsonValue(&self, i: usize) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> JsonObjectContextAttrs<'input> for JsonObjectContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonObject(&mut self,)
	-> Result<Rc<JsonObjectContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonObjectContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 144, RULE_jsonObject);
        let mut _localctx: Rc<JsonObjectContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1130);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(1134);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(112,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(1131);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(1136);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(112,&mut recog.base)?;
			}
			recog.base.set_state(1185);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_STRING {
				{
				recog.base.set_state(1137);
				recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

				recog.base.set_state(1141);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1138);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1143);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				recog.base.set_state(1144);
				recog.base.match_token(EAL_T__53,&mut recog.err_handler)?;

				recog.base.set_state(1148);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1145);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1150);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				/*InvokeRule jsonValue*/
				recog.base.set_state(1151);
				recog.jsonValue()?;

				recog.base.set_state(1182);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(119,&mut recog.base)?;
				while { _alt!=2 && _alt!=INVALID_ALT } {
					if _alt==1 {
						{
						{
						recog.base.set_state(1155);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1152);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1157);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1158);
						recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

						recog.base.set_state(1162);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1159);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1164);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1165);
						recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

						recog.base.set_state(1169);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1166);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1171);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1172);
						recog.base.match_token(EAL_T__53,&mut recog.err_handler)?;

						recog.base.set_state(1176);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1173);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1178);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						/*InvokeRule jsonValue*/
						recog.base.set_state(1179);
						recog.jsonValue()?;

						}
						} 
					}
					recog.base.set_state(1184);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(119,&mut recog.base)?;
				}
				}
			}

			recog.base.set_state(1190);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1187);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1192);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1193);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonArray ----------------
pub type JsonArrayContextAll<'input> = JsonArrayContext<'input>;


pub type JsonArrayContext<'input> = BaseParserRuleContext<'input,JsonArrayContextExt<'input>>;

#[derive(Clone)]
pub struct JsonArrayContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for JsonArrayContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for JsonArrayContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonArray(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonArray(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for JsonArrayContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_jsonArray(self);
	}
}

impl<'input> CustomRuleContext<'input> for JsonArrayContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonArray }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonArray }
}
antlr4rust::tid!{JsonArrayContextExt<'a>}

impl<'input> JsonArrayContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonArrayContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonArrayContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonArrayContextAttrs<'input>: EALParserContext<'input> + BorrowMut<JsonArrayContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
fn jsonValue_all(&self) ->  Vec<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn jsonValue(&self, i: usize) -> Option<Rc<JsonValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> JsonArrayContextAttrs<'input> for JsonArrayContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonArray(&mut self,)
	-> Result<Rc<JsonArrayContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonArrayContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 146, RULE_jsonArray);
        let mut _localctx: Rc<JsonArrayContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1195);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

			recog.base.set_state(1199);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(122,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(1196);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(1201);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(122,&mut recog.base)?;
			}
			recog.base.set_state(1222);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__3 || _la==EAL_T__45 || _la==EAL_T__71 || ((((_la - 83)) & !0x3f) == 0 && ((1usize << (_la - 83)) & 55) != 0) {
				{
				/*InvokeRule jsonValue*/
				recog.base.set_state(1202);
				recog.jsonValue()?;

				recog.base.set_state(1219);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(125,&mut recog.base)?;
				while { _alt!=2 && _alt!=INVALID_ALT } {
					if _alt==1 {
						{
						{
						recog.base.set_state(1206);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1203);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1208);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1209);
						recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

						recog.base.set_state(1213);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1210);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1215);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						/*InvokeRule jsonValue*/
						recog.base.set_state(1216);
						recog.jsonValue()?;

						}
						} 
					}
					recog.base.set_state(1221);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(125,&mut recog.base)?;
				}
				}
			}

			recog.base.set_state(1227);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1224);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1229);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1230);
			recog.base.match_token(EAL_T__46,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- jsonScalar ----------------
pub type JsonScalarContextAll<'input> = JsonScalarContext<'input>;


pub type JsonScalarContext<'input> = BaseParserRuleContext<'input,JsonScalarContextExt<'input>>;

#[derive(Clone)]
pub struct JsonScalarContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for JsonScalarContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for JsonScalarContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_jsonScalar(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_jsonScalar(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for JsonScalarContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_jsonScalar(self);
	}
}

impl<'input> CustomRuleContext<'input> for JsonScalarContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_jsonScalar }
	//fn type_rule_index() -> usize where Self: Sized { RULE_jsonScalar }
}
antlr4rust::tid!{JsonScalarContextExt<'a>}

impl<'input> JsonScalarContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<JsonScalarContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,JsonScalarContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait JsonScalarContextAttrs<'input>: EALParserContext<'input> + BorrowMut<JsonScalarContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
}
fn signedNumber(&self) -> Option<Rc<SignedNumberContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> JsonScalarContextAttrs<'input> for JsonScalarContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn jsonScalar(&mut self,)
	-> Result<Rc<JsonScalarContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = JsonScalarContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 148, RULE_jsonScalar);
        let mut _localctx: Rc<JsonScalarContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(1237);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_STRING 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1232);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__71 |EAL_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule signedNumber*/
					recog.base.set_state(1233);
					recog.signedNumber()?;

					}
				}

			EAL_T__82 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(1234);
					recog.base.match_token(EAL_T__82,&mut recog.err_handler)?;

					}
				}

			EAL_T__83 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(1235);
					recog.base.match_token(EAL_T__83,&mut recog.err_handler)?;

					}
				}

			EAL_T__84 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(1236);
					recog.base.match_token(EAL_T__84,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- signedNumber ----------------
pub type SignedNumberContextAll<'input> = SignedNumberContext<'input>;


pub type SignedNumberContext<'input> = BaseParserRuleContext<'input,SignedNumberContextExt<'input>>;

#[derive(Clone)]
pub struct SignedNumberContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for SignedNumberContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for SignedNumberContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_signedNumber(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_signedNumber(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for SignedNumberContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_signedNumber(self);
	}
}

impl<'input> CustomRuleContext<'input> for SignedNumberContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_signedNumber }
	//fn type_rule_index() -> usize where Self: Sized { RULE_signedNumber }
}
antlr4rust::tid!{SignedNumberContextExt<'a>}

impl<'input> SignedNumberContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<SignedNumberContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,SignedNumberContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait SignedNumberContextAttrs<'input>: EALParserContext<'input> + BorrowMut<SignedNumberContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NUMBER, 0)
}

}

impl<'input> SignedNumberContextAttrs<'input> for SignedNumberContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn signedNumber(&mut self,)
	-> Result<Rc<SignedNumberContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = SignedNumberContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 150, RULE_signedNumber);
        let mut _localctx: Rc<SignedNumberContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1240);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__71 {
				{
				recog.base.set_state(1239);
				recog.base.match_token(EAL_T__71,&mut recog.err_handler)?;

				}
			}

			recog.base.set_state(1242);
			recog.base.match_token(EAL_NUMBER,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- identifier ----------------
pub type IdentifierContextAll<'input> = IdentifierContext<'input>;


pub type IdentifierContext<'input> = BaseParserRuleContext<'input,IdentifierContextExt<'input>>;

#[derive(Clone)]
pub struct IdentifierContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for IdentifierContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for IdentifierContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_identifier(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_identifier(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for IdentifierContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_identifier(self);
	}
}

impl<'input> CustomRuleContext<'input> for IdentifierContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_identifier }
	//fn type_rule_index() -> usize where Self: Sized { RULE_identifier }
}
antlr4rust::tid!{IdentifierContextExt<'a>}

impl<'input> IdentifierContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<IdentifierContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,IdentifierContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait IdentifierContextAttrs<'input>: EALParserContext<'input> + BorrowMut<IdentifierContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token ID
/// Returns `None` if there is no child corresponding to token ID
fn ID(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_ID, 0)
}

}

impl<'input> IdentifierContextAttrs<'input> for IdentifierContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn identifier(&mut self,)
	-> Result<Rc<IdentifierContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = IdentifierContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 152, RULE_identifier);
        let mut _localctx: Rc<IdentifierContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(1244);
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 490602956) != 0) || ((((_la - 34)) & !0x3f) == 0 && ((1usize << (_la - 34)) & 4243130991) != 0) || _la==EAL_T__65 || _la==EAL_ID) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- lineEnd ----------------
pub type LineEndContextAll<'input> = LineEndContext<'input>;


pub type LineEndContext<'input> = BaseParserRuleContext<'input,LineEndContextExt<'input>>;

#[derive(Clone)]
pub struct LineEndContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> EALParserContext<'input> for LineEndContext<'input>{}

impl<'input,'a> Listenable<dyn EALListener<'input> + 'a> for LineEndContext<'input>{
		fn enter(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.enter_every_rule(self)?;
			listener.enter_lineEnd(self);
			Ok(())
		}
		fn exit(&self,listener: &mut (dyn EALListener<'input> + 'a)) -> Result<(), ANTLRError> {
			listener.exit_lineEnd(self);
			listener.exit_every_rule(self)?;
			Ok(())
		}
}

impl<'input,'a> Visitable<dyn EALVisitor<'input> + 'a> for LineEndContext<'input>{
	fn accept(&self,visitor: &mut (dyn EALVisitor<'input> + 'a)) {
		visitor.visit_lineEnd(self);
	}
}

impl<'input> CustomRuleContext<'input> for LineEndContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = EALParserContextType;
	fn get_rule_index(&self) -> usize { RULE_lineEnd }
	//fn type_rule_index() -> usize where Self: Sized { RULE_lineEnd }
}
antlr4rust::tid!{LineEndContextExt<'a>}

impl<'input> LineEndContextExt<'input>{
	fn new(parent: Option<Rc<dyn EALParserContext<'input> + 'input > >, invoking_state: i32) -> Rc<LineEndContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,LineEndContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait LineEndContextAttrs<'input>: EALParserContext<'input> + BorrowMut<LineEndContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token NL in current rule
fn NL_all(&self) -> Vec<Rc<TerminalNode<'input,EALParserContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token NL, starting from 0.
/// Returns `None` if number of children corresponding to token NL is less or equal than `i`.
fn NL(&self, i: usize) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_NL, i)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_EOF, 0)
}

}

impl<'input> LineEndContextAttrs<'input> for LineEndContext<'input>{}

impl<'input, I> EALParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn lineEnd(&mut self,)
	-> Result<Rc<LineEndContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = LineEndContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 154, RULE_lineEnd);
        let mut _localctx: Rc<LineEndContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			recog.base.set_state(1252);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_NL 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1247); 
					recog.err_handler.sync(&mut recog.base)?;
					_alt = 1;
					loop {
						match _alt {
						    x if x == 1=>
							{
							{
							recog.base.set_state(1246);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}

						_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
						}
						recog.base.set_state(1249); 
						recog.err_handler.sync(&mut recog.base)?;
						_alt = recog.interpreter.adaptive_predict(130,&mut recog.base)?;
						if _alt==2 || _alt==INVALID_ALT { break }
					}
					}
				}

			EAL_EOF 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(1251);
					recog.base.match_token(EAL_EOF,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
	lazy_static!{
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(
                _ATN.clone(),
                _ATN.get_decision_state(i),
                i,
            ).into())
        }
        Arc::new(dfa)
    };
	static ref _serializedATN: Vec<i32> = vec![
		4, 1, 92, 1255, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 
		7, 4, 2, 5, 7, 5, 2, 6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 
		7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2, 13, 7, 13, 2, 14, 7, 14, 2, 15, 
		7, 15, 2, 16, 7, 16, 2, 17, 7, 17, 2, 18, 7, 18, 2, 19, 7, 19, 2, 20, 
		7, 20, 2, 21, 7, 21, 2, 22, 7, 22, 2, 23, 7, 23, 2, 24, 7, 24, 2, 25, 
		7, 25, 2, 26, 7, 26, 2, 27, 7, 27, 2, 28, 7, 28, 2, 29, 7, 29, 2, 30, 
		7, 30, 2, 31, 7, 31, 2, 32, 7, 32, 2, 33, 7, 33, 2, 34, 7, 34, 2, 35, 
		7, 35, 2, 36, 7, 36, 2, 37, 7, 37, 2, 38, 7, 38, 2, 39, 7, 39, 2, 40, 
		7, 40, 2, 41, 7, 41, 2, 42, 7, 42, 2, 43, 7, 43, 2, 44, 7, 44, 2, 45, 
		7, 45, 2, 46, 7, 46, 2, 47, 7, 47, 2, 48, 7, 48, 2, 49, 7, 49, 2, 50, 
		7, 50, 2, 51, 7, 51, 2, 52, 7, 52, 2, 53, 7, 53, 2, 54, 7, 54, 2, 55, 
		7, 55, 2, 56, 7, 56, 2, 57, 7, 57, 2, 58, 7, 58, 2, 59, 7, 59, 2, 60, 
		7, 60, 2, 61, 7, 61, 2, 62, 7, 62, 2, 63, 7, 63, 2, 64, 7, 64, 2, 65, 
		7, 65, 2, 66, 7, 66, 2, 67, 7, 67, 2, 68, 7, 68, 2, 69, 7, 69, 2, 70, 
		7, 70, 2, 71, 7, 71, 2, 72, 7, 72, 2, 73, 7, 73, 2, 74, 7, 74, 2, 75, 
		7, 75, 2, 76, 7, 76, 2, 77, 7, 77, 1, 0, 5, 0, 158, 8, 0, 10, 0, 12, 0, 
		161, 9, 0, 1, 0, 1, 0, 1, 0, 1, 0, 3, 0, 167, 8, 0, 1, 0, 1, 0, 1, 0, 
		5, 0, 172, 8, 0, 10, 0, 12, 0, 175, 9, 0, 5, 0, 177, 8, 0, 10, 0, 12, 
		0, 180, 9, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 199, 8, 1, 1, 2, 1, 
		2, 1, 2, 1, 2, 5, 2, 205, 8, 2, 10, 2, 12, 2, 208, 9, 2, 1, 2, 1, 2, 5, 
		2, 212, 8, 2, 10, 2, 12, 2, 215, 9, 2, 5, 2, 217, 8, 2, 10, 2, 12, 2, 
		220, 9, 2, 1, 2, 1, 2, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 4, 1, 4, 
		1, 4, 1, 4, 5, 4, 234, 8, 4, 10, 4, 12, 4, 237, 9, 4, 1, 4, 1, 4, 5, 4, 
		241, 8, 4, 10, 4, 12, 4, 244, 9, 4, 1, 4, 1, 4, 5, 4, 248, 8, 4, 10, 4, 
		12, 4, 251, 9, 4, 5, 4, 253, 8, 4, 10, 4, 12, 4, 256, 9, 4, 1, 4, 1, 4, 
		1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 
		1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 
		1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 3, 5, 290, 8, 5, 1, 6, 1, 6, 1, 6, 
		1, 6, 5, 6, 296, 8, 6, 10, 6, 12, 6, 299, 9, 6, 1, 6, 4, 6, 302, 8, 6, 
		11, 6, 12, 6, 303, 1, 6, 1, 6, 1, 7, 1, 7, 1, 7, 1, 7, 5, 7, 312, 8, 7, 
		10, 7, 12, 7, 315, 9, 7, 1, 7, 5, 7, 318, 8, 7, 10, 7, 12, 7, 321, 9, 
		7, 1, 7, 1, 7, 1, 8, 1, 8, 1, 8, 1, 8, 1, 9, 1, 9, 1, 9, 1, 9, 5, 9, 333, 
		8, 9, 10, 9, 12, 9, 336, 9, 9, 1, 9, 5, 9, 339, 8, 9, 10, 9, 12, 9, 342, 
		9, 9, 1, 9, 1, 9, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 
		10, 3, 10, 354, 8, 10, 1, 11, 1, 11, 1, 11, 1, 11, 1, 12, 1, 12, 1, 12, 
		1, 12, 1, 13, 1, 13, 1, 13, 1, 13, 1, 14, 1, 14, 1, 14, 1, 14, 1, 15, 
		1, 15, 1, 15, 1, 15, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 
		1, 17, 1, 17, 1, 17, 1, 17, 1, 18, 1, 18, 1, 18, 1, 18, 5, 18, 391, 8, 
		18, 10, 18, 12, 18, 394, 9, 18, 1, 18, 5, 18, 397, 8, 18, 10, 18, 12, 
		18, 400, 9, 18, 1, 18, 1, 18, 1, 19, 1, 19, 1, 19, 1, 19, 1, 19, 3, 19, 
		409, 8, 19, 1, 20, 1, 20, 1, 20, 1, 20, 1, 21, 1, 21, 1, 21, 1, 21, 1, 
		22, 1, 22, 1, 22, 1, 22, 1, 23, 1, 23, 1, 23, 1, 23, 1, 24, 1, 24, 1, 
		24, 1, 24, 5, 24, 431, 8, 24, 10, 24, 12, 24, 434, 9, 24, 1, 24, 5, 24, 
		437, 8, 24, 10, 24, 12, 24, 440, 9, 24, 1, 24, 1, 24, 1, 25, 1, 25, 1, 
		25, 1, 25, 1, 25, 3, 25, 449, 8, 25, 1, 26, 1, 26, 1, 26, 1, 26, 1, 27, 
		1, 27, 1, 27, 1, 27, 1, 28, 1, 28, 1, 28, 1, 28, 1, 29, 1, 29, 1, 29, 
		1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 30, 1, 30, 
		1, 30, 1, 30, 5, 30, 478, 8, 30, 10, 30, 12, 30, 481, 9, 30, 1, 30, 5, 
		30, 484, 8, 30, 10, 30, 12, 30, 487, 9, 30, 1, 30, 1, 30, 1, 31, 1, 31, 
		1, 31, 3, 31, 494, 8, 31, 1, 32, 1, 32, 1, 32, 5, 32, 499, 8, 32, 10, 
		32, 12, 32, 502, 9, 32, 1, 32, 5, 32, 505, 8, 32, 10, 32, 12, 32, 508, 
		9, 32, 1, 32, 1, 32, 5, 32, 512, 8, 32, 10, 32, 12, 32, 515, 9, 32, 1, 
		33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 3, 33, 525, 8, 33, 
		1, 34, 1, 34, 1, 34, 1, 34, 1, 35, 1, 35, 1, 35, 1, 35, 1, 36, 1, 36, 
		1, 36, 1, 36, 1, 37, 1, 37, 1, 37, 1, 37, 1, 38, 1, 38, 1, 38, 1, 38, 
		1, 39, 1, 39, 1, 39, 1, 39, 1, 40, 1, 40, 1, 40, 1, 40, 5, 40, 555, 8, 
		40, 10, 40, 12, 40, 558, 9, 40, 1, 40, 1, 40, 1, 40, 1, 40, 1, 40, 5, 
		40, 565, 8, 40, 10, 40, 12, 40, 568, 9, 40, 1, 40, 1, 40, 5, 40, 572, 
		8, 40, 10, 40, 12, 40, 575, 9, 40, 5, 40, 577, 8, 40, 10, 40, 12, 40, 
		580, 9, 40, 1, 40, 1, 40, 1, 40, 1, 40, 3, 40, 586, 8, 40, 1, 41, 1, 41, 
		5, 41, 590, 8, 41, 10, 41, 12, 41, 593, 9, 41, 1, 41, 1, 41, 5, 41, 597, 
		8, 41, 10, 41, 12, 41, 600, 9, 41, 1, 41, 1, 41, 5, 41, 604, 8, 41, 10, 
		41, 12, 41, 607, 9, 41, 1, 41, 1, 41, 5, 41, 611, 8, 41, 10, 41, 12, 41, 
		614, 9, 41, 1, 41, 1, 41, 5, 41, 618, 8, 41, 10, 41, 12, 41, 621, 9, 41, 
		1, 41, 1, 41, 3, 41, 625, 8, 41, 1, 42, 1, 42, 5, 42, 629, 8, 42, 10, 
		42, 12, 42, 632, 9, 42, 1, 42, 1, 42, 1, 42, 5, 42, 637, 8, 42, 10, 42, 
		12, 42, 640, 9, 42, 1, 42, 5, 42, 643, 8, 42, 10, 42, 12, 42, 646, 9, 
		42, 3, 42, 648, 8, 42, 1, 42, 5, 42, 651, 8, 42, 10, 42, 12, 42, 654, 
		9, 42, 1, 42, 1, 42, 1, 43, 1, 43, 1, 43, 1, 43, 5, 43, 662, 8, 43, 10, 
		43, 12, 43, 665, 9, 43, 1, 43, 5, 43, 668, 8, 43, 10, 43, 12, 43, 671, 
		9, 43, 1, 44, 1, 44, 1, 45, 1, 45, 1, 45, 1, 45, 5, 45, 679, 8, 45, 10, 
		45, 12, 45, 682, 9, 45, 1, 45, 1, 45, 1, 45, 5, 45, 687, 8, 45, 10, 45, 
		12, 45, 690, 9, 45, 1, 45, 5, 45, 693, 8, 45, 10, 45, 12, 45, 696, 9, 
		45, 3, 45, 698, 8, 45, 1, 45, 5, 45, 701, 8, 45, 10, 45, 12, 45, 704, 
		9, 45, 1, 45, 1, 45, 5, 45, 708, 8, 45, 10, 45, 12, 45, 711, 9, 45, 1, 
		45, 1, 45, 3, 45, 715, 8, 45, 1, 45, 5, 45, 718, 8, 45, 10, 45, 12, 45, 
		721, 9, 45, 1, 45, 1, 45, 5, 45, 725, 8, 45, 10, 45, 12, 45, 728, 9, 45, 
		1, 45, 1, 45, 1, 45, 1, 45, 1, 45, 5, 45, 735, 8, 45, 10, 45, 12, 45, 
		738, 9, 45, 1, 45, 1, 45, 5, 45, 742, 8, 45, 10, 45, 12, 45, 745, 9, 45, 
		4, 45, 747, 8, 45, 11, 45, 12, 45, 748, 1, 45, 1, 45, 3, 45, 753, 8, 45, 
		1, 46, 1, 46, 1, 46, 1, 46, 1, 46, 3, 46, 760, 8, 46, 1, 47, 1, 47, 1, 
		47, 1, 47, 1, 47, 1, 47, 1, 47, 3, 47, 769, 8, 47, 1, 48, 1, 48, 1, 48, 
		1, 48, 1, 48, 1, 48, 5, 48, 777, 8, 48, 10, 48, 12, 48, 780, 9, 48, 1, 
		48, 1, 48, 1, 48, 5, 48, 785, 8, 48, 10, 48, 12, 48, 788, 9, 48, 1, 48, 
		5, 48, 791, 8, 48, 10, 48, 12, 48, 794, 9, 48, 3, 48, 796, 8, 48, 1, 48, 
		5, 48, 799, 8, 48, 10, 48, 12, 48, 802, 9, 48, 1, 48, 1, 48, 1, 48, 1, 
		49, 1, 49, 1, 49, 1, 49, 1, 50, 1, 50, 1, 50, 5, 50, 814, 8, 50, 10, 50, 
		12, 50, 817, 9, 50, 1, 50, 1, 50, 1, 50, 5, 50, 822, 8, 50, 10, 50, 12, 
		50, 825, 9, 50, 1, 50, 5, 50, 828, 8, 50, 10, 50, 12, 50, 831, 9, 50, 
		3, 50, 833, 8, 50, 1, 50, 5, 50, 836, 8, 50, 10, 50, 12, 50, 839, 9, 50, 
		1, 50, 3, 50, 842, 8, 50, 1, 51, 1, 51, 1, 51, 1, 51, 5, 51, 848, 8, 51, 
		10, 51, 12, 51, 851, 9, 51, 1, 51, 1, 51, 5, 51, 855, 8, 51, 10, 51, 12, 
		51, 858, 9, 51, 5, 51, 860, 8, 51, 10, 51, 12, 51, 863, 9, 51, 1, 51, 
		1, 51, 5, 51, 867, 8, 51, 10, 51, 12, 51, 870, 9, 51, 1, 51, 1, 51, 1, 
		51, 5, 51, 875, 8, 51, 10, 51, 12, 51, 878, 9, 51, 1, 51, 1, 51, 5, 51, 
		882, 8, 51, 10, 51, 12, 51, 885, 9, 51, 5, 51, 887, 8, 51, 10, 51, 12, 
		51, 890, 9, 51, 1, 51, 3, 51, 893, 8, 51, 1, 52, 1, 52, 1, 52, 1, 52, 
		5, 52, 899, 8, 52, 10, 52, 12, 52, 902, 9, 52, 1, 52, 1, 52, 5, 52, 906, 
		8, 52, 10, 52, 12, 52, 909, 9, 52, 1, 52, 1, 52, 5, 52, 913, 8, 52, 10, 
		52, 12, 52, 916, 9, 52, 1, 52, 1, 52, 1, 52, 1, 52, 1, 53, 1, 53, 5, 53, 
		924, 8, 53, 10, 53, 12, 53, 927, 9, 53, 1, 53, 1, 53, 1, 53, 5, 53, 932, 
		8, 53, 10, 53, 12, 53, 935, 9, 53, 1, 53, 5, 53, 938, 8, 53, 10, 53, 12, 
		53, 941, 9, 53, 3, 53, 943, 8, 53, 1, 53, 5, 53, 946, 8, 53, 10, 53, 12, 
		53, 949, 9, 53, 1, 53, 1, 53, 1, 54, 1, 54, 1, 54, 1, 54, 5, 54, 957, 
		8, 54, 10, 54, 12, 54, 960, 9, 54, 1, 54, 5, 54, 963, 8, 54, 10, 54, 12, 
		54, 966, 9, 54, 1, 55, 1, 55, 1, 56, 1, 56, 1, 57, 1, 57, 1, 57, 1, 57, 
		1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 
		1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 
		1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 3, 57, 1001, 8, 57, 1, 58, 1, 58, 5, 
		58, 1005, 8, 58, 10, 58, 12, 58, 1008, 9, 58, 1, 58, 1, 58, 1, 58, 5, 
		58, 1013, 8, 58, 10, 58, 12, 58, 1016, 9, 58, 1, 58, 5, 58, 1019, 8, 58, 
		10, 58, 12, 58, 1022, 9, 58, 1, 58, 5, 58, 1025, 8, 58, 10, 58, 12, 58, 
		1028, 9, 58, 1, 58, 1, 58, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 
		59, 1, 59, 1, 59, 1, 59, 1, 59, 3, 59, 1043, 8, 59, 1, 60, 1, 60, 1, 60, 
		5, 60, 1048, 8, 60, 10, 60, 12, 60, 1051, 9, 60, 1, 61, 1, 61, 1, 61, 
		1, 61, 1, 62, 1, 62, 1, 63, 1, 63, 1, 63, 5, 63, 1062, 8, 63, 10, 63, 
		12, 63, 1065, 9, 63, 1, 64, 1, 64, 1, 64, 5, 64, 1070, 8, 64, 10, 64, 
		12, 64, 1073, 9, 64, 1, 65, 1, 65, 1, 65, 1, 65, 3, 65, 1079, 8, 65, 1, 
		66, 1, 66, 1, 66, 5, 66, 1084, 8, 66, 10, 66, 12, 66, 1087, 9, 66, 1, 
		67, 1, 67, 1, 67, 5, 67, 1092, 8, 67, 10, 67, 12, 67, 1095, 9, 67, 1, 
		68, 1, 68, 1, 68, 3, 68, 1100, 8, 68, 1, 69, 1, 69, 1, 69, 1, 69, 1, 69, 
		5, 69, 1107, 8, 69, 10, 69, 12, 69, 1110, 9, 69, 3, 69, 1112, 8, 69, 1, 
		69, 1, 69, 1, 69, 1, 69, 1, 69, 1, 69, 1, 69, 1, 69, 3, 69, 1122, 8, 69, 
		1, 70, 1, 70, 1, 71, 1, 71, 1, 71, 3, 71, 1129, 8, 71, 1, 72, 1, 72, 5, 
		72, 1133, 8, 72, 10, 72, 12, 72, 1136, 9, 72, 1, 72, 1, 72, 5, 72, 1140, 
		8, 72, 10, 72, 12, 72, 1143, 9, 72, 1, 72, 1, 72, 5, 72, 1147, 8, 72, 
		10, 72, 12, 72, 1150, 9, 72, 1, 72, 1, 72, 5, 72, 1154, 8, 72, 10, 72, 
		12, 72, 1157, 9, 72, 1, 72, 1, 72, 5, 72, 1161, 8, 72, 10, 72, 12, 72, 
		1164, 9, 72, 1, 72, 1, 72, 5, 72, 1168, 8, 72, 10, 72, 12, 72, 1171, 9, 
		72, 1, 72, 1, 72, 5, 72, 1175, 8, 72, 10, 72, 12, 72, 1178, 9, 72, 1, 
		72, 5, 72, 1181, 8, 72, 10, 72, 12, 72, 1184, 9, 72, 3, 72, 1186, 8, 72, 
		1, 72, 5, 72, 1189, 8, 72, 10, 72, 12, 72, 1192, 9, 72, 1, 72, 1, 72, 
		1, 73, 1, 73, 5, 73, 1198, 8, 73, 10, 73, 12, 73, 1201, 9, 73, 1, 73, 
		1, 73, 5, 73, 1205, 8, 73, 10, 73, 12, 73, 1208, 9, 73, 1, 73, 1, 73, 
		5, 73, 1212, 8, 73, 10, 73, 12, 73, 1215, 9, 73, 1, 73, 5, 73, 1218, 8, 
		73, 10, 73, 12, 73, 1221, 9, 73, 3, 73, 1223, 8, 73, 1, 73, 5, 73, 1226, 
		8, 73, 10, 73, 12, 73, 1229, 9, 73, 1, 73, 1, 73, 1, 74, 1, 74, 1, 74, 
		1, 74, 1, 74, 3, 74, 1238, 8, 74, 1, 75, 3, 75, 1241, 8, 75, 1, 75, 1, 
		75, 1, 76, 1, 76, 1, 77, 4, 77, 1248, 8, 77, 11, 77, 12, 77, 1249, 1, 
		77, 3, 77, 1253, 8, 77, 1, 77, 0, 0, 78, 0, 2, 4, 6, 8, 10, 12, 14, 16, 
		18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 
		54, 56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 
		90, 92, 94, 96, 98, 100, 102, 104, 106, 108, 110, 112, 114, 116, 118, 
		120, 122, 124, 126, 128, 130, 132, 134, 136, 138, 140, 142, 144, 146, 
		148, 150, 152, 154, 0, 8, 2, 0, 25, 25, 48, 49, 2, 0, 25, 25, 49, 49, 
		5, 0, 29, 29, 31, 31, 38, 38, 41, 41, 58, 58, 1, 0, 71, 72, 1, 0, 73, 
		75, 2, 0, 71, 72, 76, 76, 1, 0, 77, 82, 14, 0, 2, 3, 6, 8, 17, 21, 24, 
		24, 26, 28, 34, 37, 39, 40, 43, 43, 45, 45, 50, 50, 53, 53, 55, 57, 60, 
		66, 86, 86, 1367, 0, 159, 1, 0, 0, 0, 2, 198, 1, 0, 0, 0, 4, 200, 1, 0, 
		0, 0, 6, 223, 1, 0, 0, 0, 8, 229, 1, 0, 0, 0, 10, 289, 1, 0, 0, 0, 12, 
		291, 1, 0, 0, 0, 14, 307, 1, 0, 0, 0, 16, 324, 1, 0, 0, 0, 18, 328, 1, 
		0, 0, 0, 20, 353, 1, 0, 0, 0, 22, 355, 1, 0, 0, 0, 24, 359, 1, 0, 0, 0, 
		26, 363, 1, 0, 0, 0, 28, 367, 1, 0, 0, 0, 30, 371, 1, 0, 0, 0, 32, 375, 
		1, 0, 0, 0, 34, 382, 1, 0, 0, 0, 36, 386, 1, 0, 0, 0, 38, 408, 1, 0, 0, 
		0, 40, 410, 1, 0, 0, 0, 42, 414, 1, 0, 0, 0, 44, 418, 1, 0, 0, 0, 46, 
		422, 1, 0, 0, 0, 48, 426, 1, 0, 0, 0, 50, 448, 1, 0, 0, 0, 52, 450, 1, 
		0, 0, 0, 54, 454, 1, 0, 0, 0, 56, 458, 1, 0, 0, 0, 58, 462, 1, 0, 0, 0, 
		60, 473, 1, 0, 0, 0, 62, 493, 1, 0, 0, 0, 64, 495, 1, 0, 0, 0, 66, 524, 
		1, 0, 0, 0, 68, 526, 1, 0, 0, 0, 70, 530, 1, 0, 0, 0, 72, 534, 1, 0, 0, 
		0, 74, 538, 1, 0, 0, 0, 76, 542, 1, 0, 0, 0, 78, 546, 1, 0, 0, 0, 80, 
		550, 1, 0, 0, 0, 82, 587, 1, 0, 0, 0, 84, 626, 1, 0, 0, 0, 86, 657, 1, 
		0, 0, 0, 88, 672, 1, 0, 0, 0, 90, 674, 1, 0, 0, 0, 92, 754, 1, 0, 0, 0, 
		94, 768, 1, 0, 0, 0, 96, 770, 1, 0, 0, 0, 98, 806, 1, 0, 0, 0, 100, 841, 
		1, 0, 0, 0, 102, 843, 1, 0, 0, 0, 104, 894, 1, 0, 0, 0, 106, 921, 1, 0, 
		0, 0, 108, 952, 1, 0, 0, 0, 110, 967, 1, 0, 0, 0, 112, 969, 1, 0, 0, 0, 
		114, 1000, 1, 0, 0, 0, 116, 1002, 1, 0, 0, 0, 118, 1042, 1, 0, 0, 0, 120, 
		1044, 1, 0, 0, 0, 122, 1052, 1, 0, 0, 0, 124, 1056, 1, 0, 0, 0, 126, 1058, 
		1, 0, 0, 0, 128, 1066, 1, 0, 0, 0, 130, 1074, 1, 0, 0, 0, 132, 1080, 1, 
		0, 0, 0, 134, 1088, 1, 0, 0, 0, 136, 1099, 1, 0, 0, 0, 138, 1121, 1, 0, 
		0, 0, 140, 1123, 1, 0, 0, 0, 142, 1128, 1, 0, 0, 0, 144, 1130, 1, 0, 0, 
		0, 146, 1195, 1, 0, 0, 0, 148, 1237, 1, 0, 0, 0, 150, 1240, 1, 0, 0, 0, 
		152, 1244, 1, 0, 0, 0, 154, 1252, 1, 0, 0, 0, 156, 158, 5, 91, 0, 0, 157, 
		156, 1, 0, 0, 0, 158, 161, 1, 0, 0, 0, 159, 157, 1, 0, 0, 0, 159, 160, 
		1, 0, 0, 0, 160, 162, 1, 0, 0, 0, 161, 159, 1, 0, 0, 0, 162, 163, 5, 1, 
		0, 0, 163, 166, 5, 88, 0, 0, 164, 165, 5, 2, 0, 0, 165, 167, 5, 88, 0, 
		0, 166, 164, 1, 0, 0, 0, 166, 167, 1, 0, 0, 0, 167, 168, 1, 0, 0, 0, 168, 
		178, 3, 154, 77, 0, 169, 173, 3, 2, 1, 0, 170, 172, 5, 91, 0, 0, 171, 
		170, 1, 0, 0, 0, 172, 175, 1, 0, 0, 0, 173, 171, 1, 0, 0, 0, 173, 174, 
		1, 0, 0, 0, 174, 177, 1, 0, 0, 0, 175, 173, 1, 0, 0, 0, 176, 169, 1, 0, 
		0, 0, 177, 180, 1, 0, 0, 0, 178, 176, 1, 0, 0, 0, 178, 179, 1, 0, 0, 0, 
		179, 181, 1, 0, 0, 0, 180, 178, 1, 0, 0, 0, 181, 182, 5, 0, 0, 1, 182, 
		1, 1, 0, 0, 0, 183, 199, 3, 8, 4, 0, 184, 199, 3, 4, 2, 0, 185, 199, 3, 
		6, 3, 0, 186, 199, 3, 12, 6, 0, 187, 199, 3, 14, 7, 0, 188, 199, 3, 18, 
		9, 0, 189, 199, 3, 36, 18, 0, 190, 199, 3, 48, 24, 0, 191, 199, 3, 60, 
		30, 0, 192, 199, 3, 80, 40, 0, 193, 199, 3, 104, 52, 0, 194, 199, 3, 90, 
		45, 0, 195, 199, 3, 96, 48, 0, 196, 199, 3, 102, 51, 0, 197, 199, 3, 114, 
		57, 0, 198, 183, 1, 0, 0, 0, 198, 184, 1, 0, 0, 0, 198, 185, 1, 0, 0, 
		0, 198, 186, 1, 0, 0, 0, 198, 187, 1, 0, 0, 0, 198, 188, 1, 0, 0, 0, 198, 
		189, 1, 0, 0, 0, 198, 190, 1, 0, 0, 0, 198, 191, 1, 0, 0, 0, 198, 192, 
		1, 0, 0, 0, 198, 193, 1, 0, 0, 0, 198, 194, 1, 0, 0, 0, 198, 195, 1, 0, 
		0, 0, 198, 196, 1, 0, 0, 0, 198, 197, 1, 0, 0, 0, 199, 3, 1, 0, 0, 0, 
		200, 201, 5, 3, 0, 0, 201, 202, 3, 120, 60, 0, 202, 206, 5, 4, 0, 0, 203, 
		205, 5, 91, 0, 0, 204, 203, 1, 0, 0, 0, 205, 208, 1, 0, 0, 0, 206, 204, 
		1, 0, 0, 0, 206, 207, 1, 0, 0, 0, 207, 218, 1, 0, 0, 0, 208, 206, 1, 0, 
		0, 0, 209, 213, 3, 2, 1, 0, 210, 212, 5, 91, 0, 0, 211, 210, 1, 0, 0, 
		0, 212, 215, 1, 0, 0, 0, 213, 211, 1, 0, 0, 0, 213, 214, 1, 0, 0, 0, 214, 
		217, 1, 0, 0, 0, 215, 213, 1, 0, 0, 0, 216, 209, 1, 0, 0, 0, 217, 220, 
		1, 0, 0, 0, 218, 216, 1, 0, 0, 0, 218, 219, 1, 0, 0, 0, 219, 221, 1, 0, 
		0, 0, 220, 218, 1, 0, 0, 0, 221, 222, 5, 5, 0, 0, 222, 5, 1, 0, 0, 0, 
		223, 224, 5, 6, 0, 0, 224, 225, 5, 88, 0, 0, 225, 226, 5, 7, 0, 0, 226, 
		227, 3, 152, 76, 0, 227, 228, 3, 154, 77, 0, 228, 7, 1, 0, 0, 0, 229, 
		230, 5, 8, 0, 0, 230, 235, 3, 10, 5, 0, 231, 232, 5, 9, 0, 0, 232, 234, 
		3, 10, 5, 0, 233, 231, 1, 0, 0, 0, 234, 237, 1, 0, 0, 0, 235, 233, 1, 
		0, 0, 0, 235, 236, 1, 0, 0, 0, 236, 238, 1, 0, 0, 0, 237, 235, 1, 0, 0, 
		0, 238, 242, 5, 4, 0, 0, 239, 241, 5, 91, 0, 0, 240, 239, 1, 0, 0, 0, 
		241, 244, 1, 0, 0, 0, 242, 240, 1, 0, 0, 0, 242, 243, 1, 0, 0, 0, 243, 
		254, 1, 0, 0, 0, 244, 242, 1, 0, 0, 0, 245, 249, 3, 2, 1, 0, 246, 248, 
		5, 91, 0, 0, 247, 246, 1, 0, 0, 0, 248, 251, 1, 0, 0, 0, 249, 247, 1, 
		0, 0, 0, 249, 250, 1, 0, 0, 0, 250, 253, 1, 0, 0, 0, 251, 249, 1, 0, 0, 
		0, 252, 245, 1, 0, 0, 0, 253, 256, 1, 0, 0, 0, 254, 252, 1, 0, 0, 0, 254, 
		255, 1, 0, 0, 0, 255, 257, 1, 0, 0, 0, 256, 254, 1, 0, 0, 0, 257, 258, 
		5, 5, 0, 0, 258, 9, 1, 0, 0, 0, 259, 260, 5, 10, 0, 0, 260, 290, 3, 120, 
		60, 0, 261, 262, 5, 11, 0, 0, 262, 290, 3, 120, 60, 0, 263, 264, 5, 12, 
		0, 0, 264, 290, 3, 150, 75, 0, 265, 266, 5, 13, 0, 0, 266, 290, 3, 152, 
		76, 0, 267, 268, 5, 14, 0, 0, 268, 290, 3, 142, 71, 0, 269, 270, 5, 15, 
		0, 0, 270, 290, 5, 88, 0, 0, 271, 272, 5, 16, 0, 0, 272, 290, 3, 120, 
		60, 0, 273, 274, 5, 17, 0, 0, 274, 290, 5, 88, 0, 0, 275, 276, 5, 18, 
		0, 0, 276, 290, 5, 88, 0, 0, 277, 278, 5, 19, 0, 0, 278, 290, 5, 88, 0, 
		0, 279, 280, 5, 20, 0, 0, 280, 290, 5, 88, 0, 0, 281, 282, 5, 21, 0, 0, 
		282, 290, 5, 88, 0, 0, 283, 284, 5, 22, 0, 0, 284, 290, 5, 88, 0, 0, 285, 
		286, 5, 23, 0, 0, 286, 290, 5, 88, 0, 0, 287, 288, 5, 24, 0, 0, 288, 290, 
		3, 142, 71, 0, 289, 259, 1, 0, 0, 0, 289, 261, 1, 0, 0, 0, 289, 263, 1, 
		0, 0, 0, 289, 265, 1, 0, 0, 0, 289, 267, 1, 0, 0, 0, 289, 269, 1, 0, 0, 
		0, 289, 271, 1, 0, 0, 0, 289, 273, 1, 0, 0, 0, 289, 275, 1, 0, 0, 0, 289, 
		277, 1, 0, 0, 0, 289, 279, 1, 0, 0, 0, 289, 281, 1, 0, 0, 0, 289, 283, 
		1, 0, 0, 0, 289, 285, 1, 0, 0, 0, 289, 287, 1, 0, 0, 0, 290, 11, 1, 0, 
		0, 0, 291, 292, 5, 10, 0, 0, 292, 293, 3, 120, 60, 0, 293, 297, 5, 4, 
		0, 0, 294, 296, 5, 91, 0, 0, 295, 294, 1, 0, 0, 0, 296, 299, 1, 0, 0, 
		0, 297, 295, 1, 0, 0, 0, 297, 298, 1, 0, 0, 0, 298, 301, 1, 0, 0, 0, 299, 
		297, 1, 0, 0, 0, 300, 302, 3, 122, 61, 0, 301, 300, 1, 0, 0, 0, 302, 303, 
		1, 0, 0, 0, 303, 301, 1, 0, 0, 0, 303, 304, 1, 0, 0, 0, 304, 305, 1, 0, 
		0, 0, 305, 306, 5, 5, 0, 0, 306, 13, 1, 0, 0, 0, 307, 308, 5, 11, 0, 0, 
		308, 309, 3, 120, 60, 0, 309, 313, 5, 4, 0, 0, 310, 312, 5, 91, 0, 0, 
		311, 310, 1, 0, 0, 0, 312, 315, 1, 0, 0, 0, 313, 311, 1, 0, 0, 0, 313, 
		314, 1, 0, 0, 0, 314, 319, 1, 0, 0, 0, 315, 313, 1, 0, 0, 0, 316, 318, 
		3, 16, 8, 0, 317, 316, 1, 0, 0, 0, 318, 321, 1, 0, 0, 0, 319, 317, 1, 
		0, 0, 0, 319, 320, 1, 0, 0, 0, 320, 322, 1, 0, 0, 0, 321, 319, 1, 0, 0, 
		0, 322, 323, 5, 5, 0, 0, 323, 15, 1, 0, 0, 0, 324, 325, 5, 15, 0, 0, 325, 
		326, 5, 88, 0, 0, 326, 327, 3, 154, 77, 0, 327, 17, 1, 0, 0, 0, 328, 329, 
		5, 25, 0, 0, 329, 330, 3, 120, 60, 0, 330, 334, 5, 4, 0, 0, 331, 333, 
		5, 91, 0, 0, 332, 331, 1, 0, 0, 0, 333, 336, 1, 0, 0, 0, 334, 332, 1, 
		0, 0, 0, 334, 335, 1, 0, 0, 0, 335, 340, 1, 0, 0, 0, 336, 334, 1, 0, 0, 
		0, 337, 339, 3, 20, 10, 0, 338, 337, 1, 0, 0, 0, 339, 342, 1, 0, 0, 0, 
		340, 338, 1, 0, 0, 0, 340, 341, 1, 0, 0, 0, 341, 343, 1, 0, 0, 0, 342, 
		340, 1, 0, 0, 0, 343, 344, 5, 5, 0, 0, 344, 19, 1, 0, 0, 0, 345, 354, 
		3, 22, 11, 0, 346, 354, 3, 34, 17, 0, 347, 354, 3, 24, 12, 0, 348, 354, 
		3, 26, 13, 0, 349, 354, 3, 28, 14, 0, 350, 354, 3, 30, 15, 0, 351, 354, 
		3, 32, 16, 0, 352, 354, 3, 122, 61, 0, 353, 345, 1, 0, 0, 0, 353, 346, 
		1, 0, 0, 0, 353, 347, 1, 0, 0, 0, 353, 348, 1, 0, 0, 0, 353, 349, 1, 0, 
		0, 0, 353, 350, 1, 0, 0, 0, 353, 351, 1, 0, 0, 0, 353, 352, 1, 0, 0, 0, 
		354, 21, 1, 0, 0, 0, 355, 356, 5, 11, 0, 0, 356, 357, 3, 120, 60, 0, 357, 
		358, 3, 154, 77, 0, 358, 23, 1, 0, 0, 0, 359, 360, 5, 13, 0, 0, 360, 361, 
		3, 152, 76, 0, 361, 362, 3, 154, 77, 0, 362, 25, 1, 0, 0, 0, 363, 364, 
		5, 10, 0, 0, 364, 365, 3, 120, 60, 0, 365, 366, 3, 154, 77, 0, 366, 27, 
		1, 0, 0, 0, 367, 368, 5, 12, 0, 0, 368, 369, 3, 150, 75, 0, 369, 370, 
		3, 154, 77, 0, 370, 29, 1, 0, 0, 0, 371, 372, 5, 14, 0, 0, 372, 373, 3, 
		142, 71, 0, 373, 374, 3, 154, 77, 0, 374, 31, 1, 0, 0, 0, 375, 376, 5, 
		26, 0, 0, 376, 377, 5, 88, 0, 0, 377, 378, 5, 27, 0, 0, 378, 379, 3, 120, 
		60, 0, 379, 380, 5, 88, 0, 0, 380, 381, 3, 154, 77, 0, 381, 33, 1, 0, 
		0, 0, 382, 383, 5, 28, 0, 0, 383, 384, 3, 124, 62, 0, 384, 385, 3, 154, 
		77, 0, 385, 35, 1, 0, 0, 0, 386, 387, 5, 29, 0, 0, 387, 388, 3, 120, 60, 
		0, 388, 392, 5, 4, 0, 0, 389, 391, 5, 91, 0, 0, 390, 389, 1, 0, 0, 0, 
		391, 394, 1, 0, 0, 0, 392, 390, 1, 0, 0, 0, 392, 393, 1, 0, 0, 0, 393, 
		398, 1, 0, 0, 0, 394, 392, 1, 0, 0, 0, 395, 397, 3, 38, 19, 0, 396, 395, 
		1, 0, 0, 0, 397, 400, 1, 0, 0, 0, 398, 396, 1, 0, 0, 0, 398, 399, 1, 0, 
		0, 0, 399, 401, 1, 0, 0, 0, 400, 398, 1, 0, 0, 0, 401, 402, 5, 5, 0, 0, 
		402, 37, 1, 0, 0, 0, 403, 409, 3, 40, 20, 0, 404, 409, 3, 26, 13, 0, 405, 
		409, 3, 42, 21, 0, 406, 409, 3, 44, 22, 0, 407, 409, 3, 46, 23, 0, 408, 
		403, 1, 0, 0, 0, 408, 404, 1, 0, 0, 0, 408, 405, 1, 0, 0, 0, 408, 406, 
		1, 0, 0, 0, 408, 407, 1, 0, 0, 0, 409, 39, 1, 0, 0, 0, 410, 411, 5, 30, 
		0, 0, 411, 412, 5, 88, 0, 0, 412, 413, 3, 154, 77, 0, 413, 41, 1, 0, 0, 
		0, 414, 415, 5, 16, 0, 0, 415, 416, 3, 120, 60, 0, 416, 417, 3, 154, 77, 
		0, 417, 43, 1, 0, 0, 0, 418, 419, 5, 22, 0, 0, 419, 420, 5, 88, 0, 0, 
		420, 421, 3, 154, 77, 0, 421, 45, 1, 0, 0, 0, 422, 423, 5, 23, 0, 0, 423, 
		424, 5, 88, 0, 0, 424, 425, 3, 154, 77, 0, 425, 47, 1, 0, 0, 0, 426, 427, 
		5, 31, 0, 0, 427, 428, 3, 120, 60, 0, 428, 432, 5, 4, 0, 0, 429, 431, 
		5, 91, 0, 0, 430, 429, 1, 0, 0, 0, 431, 434, 1, 0, 0, 0, 432, 430, 1, 
		0, 0, 0, 432, 433, 1, 0, 0, 0, 433, 438, 1, 0, 0, 0, 434, 432, 1, 0, 0, 
		0, 435, 437, 3, 50, 25, 0, 436, 435, 1, 0, 0, 0, 437, 440, 1, 0, 0, 0, 
		438, 436, 1, 0, 0, 0, 438, 439, 1, 0, 0, 0, 439, 441, 1, 0, 0, 0, 440, 
		438, 1, 0, 0, 0, 441, 442, 5, 5, 0, 0, 442, 49, 1, 0, 0, 0, 443, 449, 
		3, 52, 26, 0, 444, 449, 3, 54, 27, 0, 445, 449, 3, 56, 28, 0, 446, 449, 
		3, 58, 29, 0, 447, 449, 3, 122, 61, 0, 448, 443, 1, 0, 0, 0, 448, 444, 
		1, 0, 0, 0, 448, 445, 1, 0, 0, 0, 448, 446, 1, 0, 0, 0, 448, 447, 1, 0, 
		0, 0, 449, 51, 1, 0, 0, 0, 450, 451, 5, 17, 0, 0, 451, 452, 5, 88, 0, 
		0, 452, 453, 3, 154, 77, 0, 453, 53, 1, 0, 0, 0, 454, 455, 5, 32, 0, 0, 
		455, 456, 5, 88, 0, 0, 456, 457, 3, 154, 77, 0, 457, 55, 1, 0, 0, 0, 458, 
		459, 5, 33, 0, 0, 459, 460, 3, 116, 58, 0, 460, 461, 3, 154, 77, 0, 461, 
		57, 1, 0, 0, 0, 462, 463, 5, 34, 0, 0, 463, 464, 5, 27, 0, 0, 464, 465, 
		3, 120, 60, 0, 465, 466, 5, 35, 0, 0, 466, 467, 3, 120, 60, 0, 467, 468, 
		5, 36, 0, 0, 468, 469, 3, 120, 60, 0, 469, 470, 5, 37, 0, 0, 470, 471, 
		5, 88, 0, 0, 471, 472, 3, 154, 77, 0, 472, 59, 1, 0, 0, 0, 473, 474, 5, 
		38, 0, 0, 474, 475, 3, 120, 60, 0, 475, 479, 5, 4, 0, 0, 476, 478, 5, 
		91, 0, 0, 477, 476, 1, 0, 0, 0, 478, 481, 1, 0, 0, 0, 479, 477, 1, 0, 
		0, 0, 479, 480, 1, 0, 0, 0, 480, 485, 1, 0, 0, 0, 481, 479, 1, 0, 0, 0, 
		482, 484, 3, 62, 31, 0, 483, 482, 1, 0, 0, 0, 484, 487, 1, 0, 0, 0, 485, 
		483, 1, 0, 0, 0, 485, 486, 1, 0, 0, 0, 486, 488, 1, 0, 0, 0, 487, 485, 
		1, 0, 0, 0, 488, 489, 5, 5, 0, 0, 489, 61, 1, 0, 0, 0, 490, 494, 3, 40, 
		20, 0, 491, 494, 3, 26, 13, 0, 492, 494, 3, 64, 32, 0, 493, 490, 1, 0, 
		0, 0, 493, 491, 1, 0, 0, 0, 493, 492, 1, 0, 0, 0, 494, 63, 1, 0, 0, 0, 
		495, 496, 5, 39, 0, 0, 496, 500, 5, 4, 0, 0, 497, 499, 5, 91, 0, 0, 498, 
		497, 1, 0, 0, 0, 499, 502, 1, 0, 0, 0, 500, 498, 1, 0, 0, 0, 500, 501, 
		1, 0, 0, 0, 501, 506, 1, 0, 0, 0, 502, 500, 1, 0, 0, 0, 503, 505, 3, 66, 
		33, 0, 504, 503, 1, 0, 0, 0, 505, 508, 1, 0, 0, 0, 506, 504, 1, 0, 0, 
		0, 506, 507, 1, 0, 0, 0, 507, 509, 1, 0, 0, 0, 508, 506, 1, 0, 0, 0, 509, 
		513, 5, 5, 0, 0, 510, 512, 5, 91, 0, 0, 511, 510, 1, 0, 0, 0, 512, 515, 
		1, 0, 0, 0, 513, 511, 1, 0, 0, 0, 513, 514, 1, 0, 0, 0, 514, 65, 1, 0, 
		0, 0, 515, 513, 1, 0, 0, 0, 516, 525, 3, 68, 34, 0, 517, 525, 3, 70, 35, 
		0, 518, 525, 3, 72, 36, 0, 519, 525, 3, 74, 37, 0, 520, 525, 3, 44, 22, 
		0, 521, 525, 3, 46, 23, 0, 522, 525, 3, 76, 38, 0, 523, 525, 3, 78, 39, 
		0, 524, 516, 1, 0, 0, 0, 524, 517, 1, 0, 0, 0, 524, 518, 1, 0, 0, 0, 524, 
		519, 1, 0, 0, 0, 524, 520, 1, 0, 0, 0, 524, 521, 1, 0, 0, 0, 524, 522, 
		1, 0, 0, 0, 524, 523, 1, 0, 0, 0, 525, 67, 1, 0, 0, 0, 526, 527, 5, 18, 
		0, 0, 527, 528, 5, 88, 0, 0, 528, 529, 3, 154, 77, 0, 529, 69, 1, 0, 0, 
		0, 530, 531, 5, 19, 0, 0, 531, 532, 5, 88, 0, 0, 532, 533, 3, 154, 77, 
		0, 533, 71, 1, 0, 0, 0, 534, 535, 5, 20, 0, 0, 535, 536, 5, 88, 0, 0, 
		536, 537, 3, 154, 77, 0, 537, 73, 1, 0, 0, 0, 538, 539, 5, 21, 0, 0, 539, 
		540, 5, 88, 0, 0, 540, 541, 3, 154, 77, 0, 541, 75, 1, 0, 0, 0, 542, 543, 
		5, 24, 0, 0, 543, 544, 3, 142, 71, 0, 544, 545, 3, 154, 77, 0, 545, 77, 
		1, 0, 0, 0, 546, 547, 5, 40, 0, 0, 547, 548, 3, 124, 62, 0, 548, 549, 
		3, 154, 77, 0, 549, 79, 1, 0, 0, 0, 550, 551, 5, 41, 0, 0, 551, 585, 3, 
		120, 60, 0, 552, 556, 5, 42, 0, 0, 553, 555, 5, 91, 0, 0, 554, 553, 1, 
		0, 0, 0, 555, 558, 1, 0, 0, 0, 556, 554, 1, 0, 0, 0, 556, 557, 1, 0, 0, 
		0, 557, 559, 1, 0, 0, 0, 558, 556, 1, 0, 0, 0, 559, 560, 3, 82, 41, 0, 
		560, 561, 3, 154, 77, 0, 561, 586, 1, 0, 0, 0, 562, 566, 5, 4, 0, 0, 563, 
		565, 5, 91, 0, 0, 564, 563, 1, 0, 0, 0, 565, 568, 1, 0, 0, 0, 566, 564, 
		1, 0, 0, 0, 566, 567, 1, 0, 0, 0, 567, 578, 1, 0, 0, 0, 568, 566, 1, 0, 
		0, 0, 569, 573, 3, 2, 1, 0, 570, 572, 5, 91, 0, 0, 571, 570, 1, 0, 0, 
		0, 572, 575, 1, 0, 0, 0, 573, 571, 1, 0, 0, 0, 573, 574, 1, 0, 0, 0, 574, 
		577, 1, 0, 0, 0, 575, 573, 1, 0, 0, 0, 576, 569, 1, 0, 0, 0, 577, 580, 
		1, 0, 0, 0, 578, 576, 1, 0, 0, 0, 578, 579, 1, 0, 0, 0, 579, 581, 1, 0, 
		0, 0, 580, 578, 1, 0, 0, 0, 581, 582, 3, 82, 41, 0, 582, 583, 3, 154, 
		77, 0, 583, 584, 5, 5, 0, 0, 584, 586, 1, 0, 0, 0, 585, 552, 1, 0, 0, 
		0, 585, 562, 1, 0, 0, 0, 586, 81, 1, 0, 0, 0, 587, 591, 3, 84, 42, 0, 
		588, 590, 5, 91, 0, 0, 589, 588, 1, 0, 0, 0, 590, 593, 1, 0, 0, 0, 591, 
		589, 1, 0, 0, 0, 591, 592, 1, 0, 0, 0, 592, 594, 1, 0, 0, 0, 593, 591, 
		1, 0, 0, 0, 594, 598, 5, 43, 0, 0, 595, 597, 5, 91, 0, 0, 596, 595, 1, 
		0, 0, 0, 597, 600, 1, 0, 0, 0, 598, 596, 1, 0, 0, 0, 598, 599, 1, 0, 0, 
		0, 599, 601, 1, 0, 0, 0, 600, 598, 1, 0, 0, 0, 601, 605, 3, 118, 59, 0, 
		602, 604, 5, 91, 0, 0, 603, 602, 1, 0, 0, 0, 604, 607, 1, 0, 0, 0, 605, 
		603, 1, 0, 0, 0, 605, 606, 1, 0, 0, 0, 606, 608, 1, 0, 0, 0, 607, 605, 
		1, 0, 0, 0, 608, 612, 5, 44, 0, 0, 609, 611, 5, 91, 0, 0, 610, 609, 1, 
		0, 0, 0, 611, 614, 1, 0, 0, 0, 612, 610, 1, 0, 0, 0, 612, 613, 1, 0, 0, 
		0, 613, 615, 1, 0, 0, 0, 614, 612, 1, 0, 0, 0, 615, 624, 3, 118, 59, 0, 
		616, 618, 5, 91, 0, 0, 617, 616, 1, 0, 0, 0, 618, 621, 1, 0, 0, 0, 619, 
		617, 1, 0, 0, 0, 619, 620, 1, 0, 0, 0, 620, 622, 1, 0, 0, 0, 621, 619, 
		1, 0, 0, 0, 622, 623, 5, 45, 0, 0, 623, 625, 3, 118, 59, 0, 624, 619, 
		1, 0, 0, 0, 624, 625, 1, 0, 0, 0, 625, 83, 1, 0, 0, 0, 626, 630, 5, 46, 
		0, 0, 627, 629, 5, 91, 0, 0, 628, 627, 1, 0, 0, 0, 629, 632, 1, 0, 0, 
		0, 630, 628, 1, 0, 0, 0, 630, 631, 1, 0, 0, 0, 631, 647, 1, 0, 0, 0, 632, 
		630, 1, 0, 0, 0, 633, 644, 3, 86, 43, 0, 634, 638, 5, 9, 0, 0, 635, 637, 
		5, 91, 0, 0, 636, 635, 1, 0, 0, 0, 637, 640, 1, 0, 0, 0, 638, 636, 1, 
		0, 0, 0, 638, 639, 1, 0, 0, 0, 639, 641, 1, 0, 0, 0, 640, 638, 1, 0, 0, 
		0, 641, 643, 3, 86, 43, 0, 642, 634, 1, 0, 0, 0, 643, 646, 1, 0, 0, 0, 
		644, 642, 1, 0, 0, 0, 644, 645, 1, 0, 0, 0, 645, 648, 1, 0, 0, 0, 646, 
		644, 1, 0, 0, 0, 647, 633, 1, 0, 0, 0, 647, 648, 1, 0, 0, 0, 648, 652, 
		1, 0, 0, 0, 649, 651, 5, 91, 0, 0, 650, 649, 1, 0, 0, 0, 651, 654, 1, 
		0, 0, 0, 652, 650, 1, 0, 0, 0, 652, 653, 1, 0, 0, 0, 653, 655, 1, 0, 0, 
		0, 654, 652, 1, 0, 0, 0, 655, 656, 5, 47, 0, 0, 656, 85, 1, 0, 0, 0, 657, 
		658, 3, 88, 44, 0, 658, 669, 3, 118, 59, 0, 659, 663, 5, 9, 0, 0, 660, 
		662, 5, 91, 0, 0, 661, 660, 1, 0, 0, 0, 662, 665, 1, 0, 0, 0, 663, 661, 
		1, 0, 0, 0, 663, 664, 1, 0, 0, 0, 664, 666, 1, 0, 0, 0, 665, 663, 1, 0, 
		0, 0, 666, 668, 3, 118, 59, 0, 667, 659, 1, 0, 0, 0, 668, 671, 1, 0, 0, 
		0, 669, 667, 1, 0, 0, 0, 669, 670, 1, 0, 0, 0, 670, 87, 1, 0, 0, 0, 671, 
		669, 1, 0, 0, 0, 672, 673, 7, 0, 0, 0, 673, 89, 1, 0, 0, 0, 674, 675, 
		5, 50, 0, 0, 675, 676, 3, 120, 60, 0, 676, 680, 5, 51, 0, 0, 677, 679, 
		5, 91, 0, 0, 678, 677, 1, 0, 0, 0, 679, 682, 1, 0, 0, 0, 680, 678, 1, 
		0, 0, 0, 680, 681, 1, 0, 0, 0, 681, 697, 1, 0, 0, 0, 682, 680, 1, 0, 0, 
		0, 683, 694, 3, 92, 46, 0, 684, 688, 5, 9, 0, 0, 685, 687, 5, 91, 0, 0, 
		686, 685, 1, 0, 0, 0, 687, 690, 1, 0, 0, 0, 688, 686, 1, 0, 0, 0, 688, 
		689, 1, 0, 0, 0, 689, 691, 1, 0, 0, 0, 690, 688, 1, 0, 0, 0, 691, 693, 
		3, 92, 46, 0, 692, 684, 1, 0, 0, 0, 693, 696, 1, 0, 0, 0, 694, 692, 1, 
		0, 0, 0, 694, 695, 1, 0, 0, 0, 695, 698, 1, 0, 0, 0, 696, 694, 1, 0, 0, 
		0, 697, 683, 1, 0, 0, 0, 697, 698, 1, 0, 0, 0, 698, 702, 1, 0, 0, 0, 699, 
		701, 5, 91, 0, 0, 700, 699, 1, 0, 0, 0, 701, 704, 1, 0, 0, 0, 702, 700, 
		1, 0, 0, 0, 702, 703, 1, 0, 0, 0, 703, 705, 1, 0, 0, 0, 704, 702, 1, 0, 
		0, 0, 705, 714, 5, 52, 0, 0, 706, 708, 5, 91, 0, 0, 707, 706, 1, 0, 0, 
		0, 708, 711, 1, 0, 0, 0, 709, 707, 1, 0, 0, 0, 709, 710, 1, 0, 0, 0, 710, 
		712, 1, 0, 0, 0, 711, 709, 1, 0, 0, 0, 712, 713, 5, 53, 0, 0, 713, 715, 
		3, 152, 76, 0, 714, 709, 1, 0, 0, 0, 714, 715, 1, 0, 0, 0, 715, 719, 1, 
		0, 0, 0, 716, 718, 5, 91, 0, 0, 717, 716, 1, 0, 0, 0, 718, 721, 1, 0, 
		0, 0, 719, 717, 1, 0, 0, 0, 719, 720, 1, 0, 0, 0, 720, 752, 1, 0, 0, 0, 
		721, 719, 1, 0, 0, 0, 722, 726, 5, 42, 0, 0, 723, 725, 5, 91, 0, 0, 724, 
		723, 1, 0, 0, 0, 725, 728, 1, 0, 0, 0, 726, 724, 1, 0, 0, 0, 726, 727, 
		1, 0, 0, 0, 727, 729, 1, 0, 0, 0, 728, 726, 1, 0, 0, 0, 729, 730, 3, 82, 
		41, 0, 730, 731, 3, 154, 77, 0, 731, 753, 1, 0, 0, 0, 732, 736, 5, 4, 
		0, 0, 733, 735, 5, 91, 0, 0, 734, 733, 1, 0, 0, 0, 735, 738, 1, 0, 0, 
		0, 736, 734, 1, 0, 0, 0, 736, 737, 1, 0, 0, 0, 737, 746, 1, 0, 0, 0, 738, 
		736, 1, 0, 0, 0, 739, 743, 3, 2, 1, 0, 740, 742, 5, 91, 0, 0, 741, 740, 
		1, 0, 0, 0, 742, 745, 1, 0, 0, 0, 743, 741, 1, 0, 0, 0, 743, 744, 1, 0, 
		0, 0, 744, 747, 1, 0, 0, 0, 745, 743, 1, 0, 0, 0, 746, 739, 1, 0, 0, 0, 
		747, 748, 1, 0, 0, 0, 748, 746, 1, 0, 0, 0, 748, 749, 1, 0, 0, 0, 749, 
		750, 1, 0, 0, 0, 750, 751, 5, 5, 0, 0, 751, 753, 1, 0, 0, 0, 752, 722, 
		1, 0, 0, 0, 752, 732, 1, 0, 0, 0, 753, 91, 1, 0, 0, 0, 754, 755, 3, 152, 
		76, 0, 755, 756, 5, 54, 0, 0, 756, 759, 3, 94, 47, 0, 757, 758, 5, 46, 
		0, 0, 758, 760, 5, 47, 0, 0, 759, 757, 1, 0, 0, 0, 759, 760, 1, 0, 0, 
		0, 760, 93, 1, 0, 0, 0, 761, 769, 5, 38, 0, 0, 762, 769, 5, 31, 0, 0, 
		763, 769, 5, 25, 0, 0, 764, 769, 5, 29, 0, 0, 765, 769, 5, 10, 0, 0, 766, 
		769, 5, 11, 0, 0, 767, 769, 3, 152, 76, 0, 768, 761, 1, 0, 0, 0, 768, 
		762, 1, 0, 0, 0, 768, 763, 1, 0, 0, 0, 768, 764, 1, 0, 0, 0, 768, 765, 
		1, 0, 0, 0, 768, 766, 1, 0, 0, 0, 768, 767, 1, 0, 0, 0, 769, 95, 1, 0, 
		0, 0, 770, 771, 5, 55, 0, 0, 771, 772, 3, 120, 60, 0, 772, 773, 5, 42, 
		0, 0, 773, 774, 3, 120, 60, 0, 774, 778, 5, 51, 0, 0, 775, 777, 5, 91, 
		0, 0, 776, 775, 1, 0, 0, 0, 777, 780, 1, 0, 0, 0, 778, 776, 1, 0, 0, 0, 
		778, 779, 1, 0, 0, 0, 779, 795, 1, 0, 0, 0, 780, 778, 1, 0, 0, 0, 781, 
		792, 3, 98, 49, 0, 782, 786, 5, 9, 0, 0, 783, 785, 5, 91, 0, 0, 784, 783, 
		1, 0, 0, 0, 785, 788, 1, 0, 0, 0, 786, 784, 1, 0, 0, 0, 786, 787, 1, 0, 
		0, 0, 787, 789, 1, 0, 0, 0, 788, 786, 1, 0, 0, 0, 789, 791, 3, 98, 49, 
		0, 790, 782, 1, 0, 0, 0, 791, 794, 1, 0, 0, 0, 792, 790, 1, 0, 0, 0, 792, 
		793, 1, 0, 0, 0, 793, 796, 1, 0, 0, 0, 794, 792, 1, 0, 0, 0, 795, 781, 
		1, 0, 0, 0, 795, 796, 1, 0, 0, 0, 796, 800, 1, 0, 0, 0, 797, 799, 5, 91, 
		0, 0, 798, 797, 1, 0, 0, 0, 799, 802, 1, 0, 0, 0, 800, 798, 1, 0, 0, 0, 
		800, 801, 1, 0, 0, 0, 801, 803, 1, 0, 0, 0, 802, 800, 1, 0, 0, 0, 803, 
		804, 5, 52, 0, 0, 804, 805, 3, 154, 77, 0, 805, 97, 1, 0, 0, 0, 806, 807, 
		3, 152, 76, 0, 807, 808, 5, 42, 0, 0, 808, 809, 3, 100, 50, 0, 809, 99, 
		1, 0, 0, 0, 810, 842, 3, 118, 59, 0, 811, 815, 5, 46, 0, 0, 812, 814, 
		5, 91, 0, 0, 813, 812, 1, 0, 0, 0, 814, 817, 1, 0, 0, 0, 815, 813, 1, 
		0, 0, 0, 815, 816, 1, 0, 0, 0, 816, 832, 1, 0, 0, 0, 817, 815, 1, 0, 0, 
		0, 818, 829, 3, 118, 59, 0, 819, 823, 5, 9, 0, 0, 820, 822, 5, 91, 0, 
		0, 821, 820, 1, 0, 0, 0, 822, 825, 1, 0, 0, 0, 823, 821, 1, 0, 0, 0, 823, 
		824, 1, 0, 0, 0, 824, 826, 1, 0, 0, 0, 825, 823, 1, 0, 0, 0, 826, 828, 
		3, 118, 59, 0, 827, 819, 1, 0, 0, 0, 828, 831, 1, 0, 0, 0, 829, 827, 1, 
		0, 0, 0, 829, 830, 1, 0, 0, 0, 830, 833, 1, 0, 0, 0, 831, 829, 1, 0, 0, 
		0, 832, 818, 1, 0, 0, 0, 832, 833, 1, 0, 0, 0, 833, 837, 1, 0, 0, 0, 834, 
		836, 5, 91, 0, 0, 835, 834, 1, 0, 0, 0, 836, 839, 1, 0, 0, 0, 837, 835, 
		1, 0, 0, 0, 837, 838, 1, 0, 0, 0, 838, 840, 1, 0, 0, 0, 839, 837, 1, 0, 
		0, 0, 840, 842, 5, 47, 0, 0, 841, 810, 1, 0, 0, 0, 841, 811, 1, 0, 0, 
		0, 842, 101, 1, 0, 0, 0, 843, 844, 5, 56, 0, 0, 844, 845, 3, 152, 76, 
		0, 845, 849, 5, 4, 0, 0, 846, 848, 5, 91, 0, 0, 847, 846, 1, 0, 0, 0, 
		848, 851, 1, 0, 0, 0, 849, 847, 1, 0, 0, 0, 849, 850, 1, 0, 0, 0, 850, 
		861, 1, 0, 0, 0, 851, 849, 1, 0, 0, 0, 852, 856, 3, 2, 1, 0, 853, 855, 
		5, 91, 0, 0, 854, 853, 1, 0, 0, 0, 855, 858, 1, 0, 0, 0, 856, 854, 1, 
		0, 0, 0, 856, 857, 1, 0, 0, 0, 857, 860, 1, 0, 0, 0, 858, 856, 1, 0, 0, 
		0, 859, 852, 1, 0, 0, 0, 860, 863, 1, 0, 0, 0, 861, 859, 1, 0, 0, 0, 861, 
		862, 1, 0, 0, 0, 862, 864, 1, 0, 0, 0, 863, 861, 1, 0, 0, 0, 864, 892, 
		5, 5, 0, 0, 865, 867, 5, 91, 0, 0, 866, 865, 1, 0, 0, 0, 867, 870, 1, 
		0, 0, 0, 868, 866, 1, 0, 0, 0, 868, 869, 1, 0, 0, 0, 869, 871, 1, 0, 0, 
		0, 870, 868, 1, 0, 0, 0, 871, 872, 5, 57, 0, 0, 872, 876, 5, 4, 0, 0, 
		873, 875, 5, 91, 0, 0, 874, 873, 1, 0, 0, 0, 875, 878, 1, 0, 0, 0, 876, 
		874, 1, 0, 0, 0, 876, 877, 1, 0, 0, 0, 877, 888, 1, 0, 0, 0, 878, 876, 
		1, 0, 0, 0, 879, 883, 3, 2, 1, 0, 880, 882, 5, 91, 0, 0, 881, 880, 1, 
		0, 0, 0, 882, 885, 1, 0, 0, 0, 883, 881, 1, 0, 0, 0, 883, 884, 1, 0, 0, 
		0, 884, 887, 1, 0, 0, 0, 885, 883, 1, 0, 0, 0, 886, 879, 1, 0, 0, 0, 887, 
		890, 1, 0, 0, 0, 888, 886, 1, 0, 0, 0, 888, 889, 1, 0, 0, 0, 889, 891, 
		1, 0, 0, 0, 890, 888, 1, 0, 0, 0, 891, 893, 5, 5, 0, 0, 892, 868, 1, 0, 
		0, 0, 892, 893, 1, 0, 0, 0, 893, 103, 1, 0, 0, 0, 894, 895, 5, 58, 0, 
		0, 895, 896, 3, 120, 60, 0, 896, 900, 5, 42, 0, 0, 897, 899, 5, 91, 0, 
		0, 898, 897, 1, 0, 0, 0, 899, 902, 1, 0, 0, 0, 900, 898, 1, 0, 0, 0, 900, 
		901, 1, 0, 0, 0, 901, 903, 1, 0, 0, 0, 902, 900, 1, 0, 0, 0, 903, 907, 
		3, 106, 53, 0, 904, 906, 5, 91, 0, 0, 905, 904, 1, 0, 0, 0, 906, 909, 
		1, 0, 0, 0, 907, 905, 1, 0, 0, 0, 907, 908, 1, 0, 0, 0, 908, 910, 1, 0, 
		0, 0, 909, 907, 1, 0, 0, 0, 910, 914, 5, 59, 0, 0, 911, 913, 5, 91, 0, 
		0, 912, 911, 1, 0, 0, 0, 913, 916, 1, 0, 0, 0, 914, 912, 1, 0, 0, 0, 914, 
		915, 1, 0, 0, 0, 915, 917, 1, 0, 0, 0, 916, 914, 1, 0, 0, 0, 917, 918, 
		3, 112, 56, 0, 918, 919, 3, 118, 59, 0, 919, 920, 3, 154, 77, 0, 920, 
		105, 1, 0, 0, 0, 921, 925, 5, 46, 0, 0, 922, 924, 5, 91, 0, 0, 923, 922, 
		1, 0, 0, 0, 924, 927, 1, 0, 0, 0, 925, 923, 1, 0, 0, 0, 925, 926, 1, 0, 
		0, 0, 926, 942, 1, 0, 0, 0, 927, 925, 1, 0, 0, 0, 928, 939, 3, 108, 54, 
		0, 929, 933, 5, 9, 0, 0, 930, 932, 5, 91, 0, 0, 931, 930, 1, 0, 0, 0, 
		932, 935, 1, 0, 0, 0, 933, 931, 1, 0, 0, 0, 933, 934, 1, 0, 0, 0, 934, 
		936, 1, 0, 0, 0, 935, 933, 1, 0, 0, 0, 936, 938, 3, 108, 54, 0, 937, 929, 
		1, 0, 0, 0, 938, 941, 1, 0, 0, 0, 939, 937, 1, 0, 0, 0, 939, 940, 1, 0, 
		0, 0, 940, 943, 1, 0, 0, 0, 941, 939, 1, 0, 0, 0, 942, 928, 1, 0, 0, 0, 
		942, 943, 1, 0, 0, 0, 943, 947, 1, 0, 0, 0, 944, 946, 5, 91, 0, 0, 945, 
		944, 1, 0, 0, 0, 946, 949, 1, 0, 0, 0, 947, 945, 1, 0, 0, 0, 947, 948, 
		1, 0, 0, 0, 948, 950, 1, 0, 0, 0, 949, 947, 1, 0, 0, 0, 950, 951, 5, 47, 
		0, 0, 951, 107, 1, 0, 0, 0, 952, 953, 3, 110, 55, 0, 953, 964, 3, 118, 
		59, 0, 954, 958, 5, 9, 0, 0, 955, 957, 5, 91, 0, 0, 956, 955, 1, 0, 0, 
		0, 957, 960, 1, 0, 0, 0, 958, 956, 1, 0, 0, 0, 958, 959, 1, 0, 0, 0, 959, 
		961, 1, 0, 0, 0, 960, 958, 1, 0, 0, 0, 961, 963, 3, 118, 59, 0, 962, 954, 
		1, 0, 0, 0, 963, 966, 1, 0, 0, 0, 964, 962, 1, 0, 0, 0, 964, 965, 1, 0, 
		0, 0, 965, 109, 1, 0, 0, 0, 966, 964, 1, 0, 0, 0, 967, 968, 7, 1, 0, 0, 
		968, 111, 1, 0, 0, 0, 969, 970, 7, 2, 0, 0, 970, 113, 1, 0, 0, 0, 971, 
		972, 5, 60, 0, 0, 972, 973, 3, 120, 60, 0, 973, 974, 5, 37, 0, 0, 974, 
		975, 5, 88, 0, 0, 975, 976, 3, 154, 77, 0, 976, 1001, 1, 0, 0, 0, 977, 
		978, 5, 61, 0, 0, 978, 979, 3, 120, 60, 0, 979, 980, 3, 150, 75, 0, 980, 
		981, 5, 37, 0, 0, 981, 982, 5, 88, 0, 0, 982, 983, 3, 154, 77, 0, 983, 
		1001, 1, 0, 0, 0, 984, 985, 5, 62, 0, 0, 985, 986, 3, 120, 60, 0, 986, 
		987, 5, 35, 0, 0, 987, 988, 3, 120, 60, 0, 988, 989, 5, 37, 0, 0, 989, 
		990, 5, 88, 0, 0, 990, 991, 3, 154, 77, 0, 991, 1001, 1, 0, 0, 0, 992, 
		993, 5, 63, 0, 0, 993, 994, 3, 120, 60, 0, 994, 995, 5, 64, 0, 0, 995, 
		996, 3, 120, 60, 0, 996, 997, 5, 37, 0, 0, 997, 998, 5, 88, 0, 0, 998, 
		999, 3, 154, 77, 0, 999, 1001, 1, 0, 0, 0, 1000, 971, 1, 0, 0, 0, 1000, 
		977, 1, 0, 0, 0, 1000, 984, 1, 0, 0, 0, 1000, 992, 1, 0, 0, 0, 1001, 115, 
		1, 0, 0, 0, 1002, 1006, 5, 46, 0, 0, 1003, 1005, 5, 91, 0, 0, 1004, 1003, 
		1, 0, 0, 0, 1005, 1008, 1, 0, 0, 0, 1006, 1004, 1, 0, 0, 0, 1006, 1007, 
		1, 0, 0, 0, 1007, 1009, 1, 0, 0, 0, 1008, 1006, 1, 0, 0, 0, 1009, 1020, 
		3, 118, 59, 0, 1010, 1014, 5, 9, 0, 0, 1011, 1013, 5, 91, 0, 0, 1012, 
		1011, 1, 0, 0, 0, 1013, 1016, 1, 0, 0, 0, 1014, 1012, 1, 0, 0, 0, 1014, 
		1015, 1, 0, 0, 0, 1015, 1017, 1, 0, 0, 0, 1016, 1014, 1, 0, 0, 0, 1017, 
		1019, 3, 118, 59, 0, 1018, 1010, 1, 0, 0, 0, 1019, 1022, 1, 0, 0, 0, 1020, 
		1018, 1, 0, 0, 0, 1020, 1021, 1, 0, 0, 0, 1021, 1026, 1, 0, 0, 0, 1022, 
		1020, 1, 0, 0, 0, 1023, 1025, 5, 91, 0, 0, 1024, 1023, 1, 0, 0, 0, 1025, 
		1028, 1, 0, 0, 0, 1026, 1024, 1, 0, 0, 0, 1026, 1027, 1, 0, 0, 0, 1027, 
		1029, 1, 0, 0, 0, 1028, 1026, 1, 0, 0, 0, 1029, 1030, 5, 47, 0, 0, 1030, 
		117, 1, 0, 0, 0, 1031, 1043, 3, 120, 60, 0, 1032, 1033, 5, 65, 0, 0, 1033, 
		1034, 5, 51, 0, 0, 1034, 1035, 3, 120, 60, 0, 1035, 1036, 5, 52, 0, 0, 
		1036, 1043, 1, 0, 0, 0, 1037, 1038, 5, 66, 0, 0, 1038, 1039, 5, 51, 0, 
		0, 1039, 1040, 3, 120, 60, 0, 1040, 1041, 5, 52, 0, 0, 1041, 1043, 1, 
		0, 0, 0, 1042, 1031, 1, 0, 0, 0, 1042, 1032, 1, 0, 0, 0, 1042, 1037, 1, 
		0, 0, 0, 1043, 119, 1, 0, 0, 0, 1044, 1049, 3, 152, 76, 0, 1045, 1046, 
		5, 67, 0, 0, 1046, 1048, 3, 152, 76, 0, 1047, 1045, 1, 0, 0, 0, 1048, 
		1051, 1, 0, 0, 0, 1049, 1047, 1, 0, 0, 0, 1049, 1050, 1, 0, 0, 0, 1050, 
		121, 1, 0, 0, 0, 1051, 1049, 1, 0, 0, 0, 1052, 1053, 5, 68, 0, 0, 1053, 
		1054, 3, 124, 62, 0, 1054, 1055, 3, 154, 77, 0, 1055, 123, 1, 0, 0, 0, 
		1056, 1057, 3, 126, 63, 0, 1057, 125, 1, 0, 0, 0, 1058, 1063, 3, 128, 
		64, 0, 1059, 1060, 5, 69, 0, 0, 1060, 1062, 3, 128, 64, 0, 1061, 1059, 
		1, 0, 0, 0, 1062, 1065, 1, 0, 0, 0, 1063, 1061, 1, 0, 0, 0, 1063, 1064, 
		1, 0, 0, 0, 1064, 127, 1, 0, 0, 0, 1065, 1063, 1, 0, 0, 0, 1066, 1071, 
		3, 130, 65, 0, 1067, 1068, 5, 70, 0, 0, 1068, 1070, 3, 130, 65, 0, 1069, 
		1067, 1, 0, 0, 0, 1070, 1073, 1, 0, 0, 0, 1071, 1069, 1, 0, 0, 0, 1071, 
		1072, 1, 0, 0, 0, 1072, 129, 1, 0, 0, 0, 1073, 1071, 1, 0, 0, 0, 1074, 
		1078, 3, 132, 66, 0, 1075, 1076, 3, 140, 70, 0, 1076, 1077, 3, 132, 66, 
		0, 1077, 1079, 1, 0, 0, 0, 1078, 1075, 1, 0, 0, 0, 1078, 1079, 1, 0, 0, 
		0, 1079, 131, 1, 0, 0, 0, 1080, 1085, 3, 134, 67, 0, 1081, 1082, 7, 3, 
		0, 0, 1082, 1084, 3, 134, 67, 0, 1083, 1081, 1, 0, 0, 0, 1084, 1087, 1, 
		0, 0, 0, 1085, 1083, 1, 0, 0, 0, 1085, 1086, 1, 0, 0, 0, 1086, 133, 1, 
		0, 0, 0, 1087, 1085, 1, 0, 0, 0, 1088, 1093, 3, 136, 68, 0, 1089, 1090, 
		7, 4, 0, 0, 1090, 1092, 3, 136, 68, 0, 1091, 1089, 1, 0, 0, 0, 1092, 1095, 
		1, 0, 0, 0, 1093, 1091, 1, 0, 0, 0, 1093, 1094, 1, 0, 0, 0, 1094, 135, 
		1, 0, 0, 0, 1095, 1093, 1, 0, 0, 0, 1096, 1097, 7, 5, 0, 0, 1097, 1100, 
		3, 136, 68, 0, 1098, 1100, 3, 138, 69, 0, 1099, 1096, 1, 0, 0, 0, 1099, 
		1098, 1, 0, 0, 0, 1100, 137, 1, 0, 0, 0, 1101, 1102, 3, 120, 60, 0, 1102, 
		1111, 5, 51, 0, 0, 1103, 1108, 3, 124, 62, 0, 1104, 1105, 5, 9, 0, 0, 
		1105, 1107, 3, 124, 62, 0, 1106, 1104, 1, 0, 0, 0, 1107, 1110, 1, 0, 0, 
		0, 1108, 1106, 1, 0, 0, 0, 1108, 1109, 1, 0, 0, 0, 1109, 1112, 1, 0, 0, 
		0, 1110, 1108, 1, 0, 0, 0, 1111, 1103, 1, 0, 0, 0, 1111, 1112, 1, 0, 0, 
		0, 1112, 1113, 1, 0, 0, 0, 1113, 1114, 5, 52, 0, 0, 1114, 1122, 1, 0, 
		0, 0, 1115, 1122, 3, 120, 60, 0, 1116, 1122, 3, 142, 71, 0, 1117, 1118, 
		5, 51, 0, 0, 1118, 1119, 3, 124, 62, 0, 1119, 1120, 5, 52, 0, 0, 1120, 
		1122, 1, 0, 0, 0, 1121, 1101, 1, 0, 0, 0, 1121, 1115, 1, 0, 0, 0, 1121, 
		1116, 1, 0, 0, 0, 1121, 1117, 1, 0, 0, 0, 1122, 139, 1, 0, 0, 0, 1123, 
		1124, 7, 6, 0, 0, 1124, 141, 1, 0, 0, 0, 1125, 1129, 3, 148, 74, 0, 1126, 
		1129, 3, 144, 72, 0, 1127, 1129, 3, 146, 73, 0, 1128, 1125, 1, 0, 0, 0, 
		1128, 1126, 1, 0, 0, 0, 1128, 1127, 1, 0, 0, 0, 1129, 143, 1, 0, 0, 0, 
		1130, 1134, 5, 4, 0, 0, 1131, 1133, 5, 91, 0, 0, 1132, 1131, 1, 0, 0, 
		0, 1133, 1136, 1, 0, 0, 0, 1134, 1132, 1, 0, 0, 0, 1134, 1135, 1, 0, 0, 
		0, 1135, 1185, 1, 0, 0, 0, 1136, 1134, 1, 0, 0, 0, 1137, 1141, 5, 88, 
		0, 0, 1138, 1140, 5, 91, 0, 0, 1139, 1138, 1, 0, 0, 0, 1140, 1143, 1, 
		0, 0, 0, 1141, 1139, 1, 0, 0, 0, 1141, 1142, 1, 0, 0, 0, 1142, 1144, 1, 
		0, 0, 0, 1143, 1141, 1, 0, 0, 0, 1144, 1148, 5, 54, 0, 0, 1145, 1147, 
		5, 91, 0, 0, 1146, 1145, 1, 0, 0, 0, 1147, 1150, 1, 0, 0, 0, 1148, 1146, 
		1, 0, 0, 0, 1148, 1149, 1, 0, 0, 0, 1149, 1151, 1, 0, 0, 0, 1150, 1148, 
		1, 0, 0, 0, 1151, 1182, 3, 142, 71, 0, 1152, 1154, 5, 91, 0, 0, 1153, 
		1152, 1, 0, 0, 0, 1154, 1157, 1, 0, 0, 0, 1155, 1153, 1, 0, 0, 0, 1155, 
		1156, 1, 0, 0, 0, 1156, 1158, 1, 0, 0, 0, 1157, 1155, 1, 0, 0, 0, 1158, 
		1162, 5, 9, 0, 0, 1159, 1161, 5, 91, 0, 0, 1160, 1159, 1, 0, 0, 0, 1161, 
		1164, 1, 0, 0, 0, 1162, 1160, 1, 0, 0, 0, 1162, 1163, 1, 0, 0, 0, 1163, 
		1165, 1, 0, 0, 0, 1164, 1162, 1, 0, 0, 0, 1165, 1169, 5, 88, 0, 0, 1166, 
		1168, 5, 91, 0, 0, 1167, 1166, 1, 0, 0, 0, 1168, 1171, 1, 0, 0, 0, 1169, 
		1167, 1, 0, 0, 0, 1169, 1170, 1, 0, 0, 0, 1170, 1172, 1, 0, 0, 0, 1171, 
		1169, 1, 0, 0, 0, 1172, 1176, 5, 54, 0, 0, 1173, 1175, 5, 91, 0, 0, 1174, 
		1173, 1, 0, 0, 0, 1175, 1178, 1, 0, 0, 0, 1176, 1174, 1, 0, 0, 0, 1176, 
		1177, 1, 0, 0, 0, 1177, 1179, 1, 0, 0, 0, 1178, 1176, 1, 0, 0, 0, 1179, 
		1181, 3, 142, 71, 0, 1180, 1155, 1, 0, 0, 0, 1181, 1184, 1, 0, 0, 0, 1182, 
		1180, 1, 0, 0, 0, 1182, 1183, 1, 0, 0, 0, 1183, 1186, 1, 0, 0, 0, 1184, 
		1182, 1, 0, 0, 0, 1185, 1137, 1, 0, 0, 0, 1185, 1186, 1, 0, 0, 0, 1186, 
		1190, 1, 0, 0, 0, 1187, 1189, 5, 91, 0, 0, 1188, 1187, 1, 0, 0, 0, 1189, 
		1192, 1, 0, 0, 0, 1190, 1188, 1, 0, 0, 0, 1190, 1191, 1, 0, 0, 0, 1191, 
		1193, 1, 0, 0, 0, 1192, 1190, 1, 0, 0, 0, 1193, 1194, 5, 5, 0, 0, 1194, 
		145, 1, 0, 0, 0, 1195, 1199, 5, 46, 0, 0, 1196, 1198, 5, 91, 0, 0, 1197, 
		1196, 1, 0, 0, 0, 1198, 1201, 1, 0, 0, 0, 1199, 1197, 1, 0, 0, 0, 1199, 
		1200, 1, 0, 0, 0, 1200, 1222, 1, 0, 0, 0, 1201, 1199, 1, 0, 0, 0, 1202, 
		1219, 3, 142, 71, 0, 1203, 1205, 5, 91, 0, 0, 1204, 1203, 1, 0, 0, 0, 
		1205, 1208, 1, 0, 0, 0, 1206, 1204, 1, 0, 0, 0, 1206, 1207, 1, 0, 0, 0, 
		1207, 1209, 1, 0, 0, 0, 1208, 1206, 1, 0, 0, 0, 1209, 1213, 5, 9, 0, 0, 
		1210, 1212, 5, 91, 0, 0, 1211, 1210, 1, 0, 0, 0, 1212, 1215, 1, 0, 0, 
		0, 1213, 1211, 1, 0, 0, 0, 1213, 1214, 1, 0, 0, 0, 1214, 1216, 1, 0, 0, 
		0, 1215, 1213, 1, 0, 0, 0, 1216, 1218, 3, 142, 71, 0, 1217, 1206, 1, 0, 
		0, 0, 1218, 1221, 1, 0, 0, 0, 1219, 1217, 1, 0, 0, 0, 1219, 1220, 1, 0, 
		0, 0, 1220, 1223, 1, 0, 0, 0, 1221, 1219, 1, 0, 0, 0, 1222, 1202, 1, 0, 
		0, 0, 1222, 1223, 1, 0, 0, 0, 1223, 1227, 1, 0, 0, 0, 1224, 1226, 5, 91, 
		0, 0, 1225, 1224, 1, 0, 0, 0, 1226, 1229, 1, 0, 0, 0, 1227, 1225, 1, 0, 
		0, 0, 1227, 1228, 1, 0, 0, 0, 1228, 1230, 1, 0, 0, 0, 1229, 1227, 1, 0, 
		0, 0, 1230, 1231, 5, 47, 0, 0, 1231, 147, 1, 0, 0, 0, 1232, 1238, 5, 88, 
		0, 0, 1233, 1238, 3, 150, 75, 0, 1234, 1238, 5, 83, 0, 0, 1235, 1238, 
		5, 84, 0, 0, 1236, 1238, 5, 85, 0, 0, 1237, 1232, 1, 0, 0, 0, 1237, 1233, 
		1, 0, 0, 0, 1237, 1234, 1, 0, 0, 0, 1237, 1235, 1, 0, 0, 0, 1237, 1236, 
		1, 0, 0, 0, 1238, 149, 1, 0, 0, 0, 1239, 1241, 5, 72, 0, 0, 1240, 1239, 
		1, 0, 0, 0, 1240, 1241, 1, 0, 0, 0, 1241, 1242, 1, 0, 0, 0, 1242, 1243, 
		5, 87, 0, 0, 1243, 151, 1, 0, 0, 0, 1244, 1245, 7, 7, 0, 0, 1245, 153, 
		1, 0, 0, 0, 1246, 1248, 5, 91, 0, 0, 1247, 1246, 1, 0, 0, 0, 1248, 1249, 
		1, 0, 0, 0, 1249, 1247, 1, 0, 0, 0, 1249, 1250, 1, 0, 0, 0, 1250, 1253, 
		1, 0, 0, 0, 1251, 1253, 5, 0, 0, 1, 1252, 1247, 1, 0, 0, 0, 1252, 1251, 
		1, 0, 0, 0, 1253, 155, 1, 0, 0, 0, 132, 159, 166, 173, 178, 198, 206, 
		213, 218, 235, 242, 249, 254, 289, 297, 303, 313, 319, 334, 340, 353, 
		392, 398, 408, 432, 438, 448, 479, 485, 493, 500, 506, 513, 524, 556, 
		566, 573, 578, 585, 591, 598, 605, 612, 619, 624, 630, 638, 644, 647, 
		652, 663, 669, 680, 688, 694, 697, 702, 709, 714, 719, 726, 736, 743, 
		748, 752, 759, 768, 778, 786, 792, 795, 800, 815, 823, 829, 832, 837, 
		841, 849, 856, 861, 868, 876, 883, 888, 892, 900, 907, 914, 925, 933, 
		939, 942, 947, 958, 964, 1000, 1006, 1014, 1020, 1026, 1042, 1049, 1063, 
		1071, 1078, 1085, 1093, 1099, 1108, 1111, 1121, 1128, 1134, 1141, 1148, 
		1155, 1162, 1169, 1176, 1182, 1185, 1190, 1199, 1206, 1213, 1219, 1222, 
		1227, 1237, 1240, 1249, 1252
	];
}
