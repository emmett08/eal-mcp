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
		pub const EAL_ID:i32=85; 
		pub const EAL_NUMBER:i32=86; 
		pub const EAL_STRING:i32=87; 
		pub const EAL_LINE_COMMENT:i32=88; 
		pub const EAL_BLOCK_COMMENT:i32=89; 
		pub const EAL_NL:i32=90; 
		pub const EAL_WS:i32=91;
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


	pub const _LITERAL_NAMES: [Option<&'static str>;85] = [
		None, Some("'language'"), Some("'module'"), Some("'{'"), Some("'}'"), 
		Some("'import'"), Some("'as'"), Some("'context'"), Some("','"), Some("'environment'"), 
		Some("'tool'"), Some("'max_age'"), Some("'kind'"), Some("'input'"), Some("'version'"), 
		Some("'validate'"), Some("'method'"), Some("'subject'"), Some("'quantity'"), 
		Some("'unit'"), Some("'scope'"), Some("'valid_from'"), Some("'valid_until'"), 
		Some("'query'"), Some("'evidence'"), Some("'bind'"), Some("'from'"), Some("'derive'"), 
		Some("'assumption'"), Some("'statement'"), Some("'reasoning'"), Some("'rationale'"), 
		Some("'backing'"), Some("'transfer'"), Some("'to'"), Some("'assuming'"), 
		Some("'reviewed'"), Some("'claim'"), Some("'proposition'"), Some("'result'"), 
		Some("'argument'"), Some("'='"), Some("'via'"), Some("'=>'"), Some("'binding'"), 
		Some("'['"), Some("']'"), Some("'assumptions'"), Some("'premises'"), Some("'pattern'"), 
		Some("'('"), Some("')'"), Some("'decreases'"), Some("':'"), Some("'apply'"), 
		Some("'when'"), Some("'else'"), Some("'objection'"), Some("'-x>'"), Some("'strict'"), 
		Some("'rank'"), Some("'contrary'"), Some("'prefer'"), Some("'over'"), 
		Some("'head'"), Some("'tail'"), Some("'.'"), Some("'require'"), Some("'or'"), 
		Some("'and'"), Some("'+'"), Some("'-'"), Some("'*'"), Some("'/'"), Some("'%'"), 
		Some("'not'"), Some("'=='"), Some("'!='"), Some("'<='"), Some("'>='"), 
		Some("'<'"), Some("'>'"), Some("'true'"), Some("'false'"), Some("'null'")
	];
	pub const _SYMBOLIC_NAMES: [Option<&'static str>;92]  = [
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, None, None, None, None, None, None, None, None, None, None, None, 
		None, Some("ID"), Some("NUMBER"), Some("STRING"), Some("LINE_COMMENT"), 
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

/// Retrieves first TerminalNode corresponding to token STRING
/// Returns `None` if there is no child corresponding to token STRING
fn STRING(&self) -> Option<Rc<TerminalNode<'input,EALParserContextType>>> where Self:Sized{
	self.get_token(EAL_STRING, 0)
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

			/*InvokeRule lineEnd*/
			recog.base.set_state(164);
			recog.lineEnd()?;

			recog.base.set_state(174);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(165);
				recog.declaration()?;

				recog.base.set_state(169);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(166);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(171);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(176);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(177);
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

			recog.base.set_state(194);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__6 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule contextDecl*/
					recog.base.set_state(179);
					recog.contextDecl()?;

					}
				}

			EAL_T__1 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule moduleDecl*/
					recog.base.set_state(180);
					recog.moduleDecl()?;

					}
				}

			EAL_T__4 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule importDecl*/
					recog.base.set_state(181);
					recog.importDecl()?;

					}
				}

			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule environmentDecl*/
					recog.base.set_state(182);
					recog.environmentDecl()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule toolDecl*/
					recog.base.set_state(183);
					recog.toolDecl()?;

					}
				}

			EAL_T__23 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule evidenceDecl*/
					recog.base.set_state(184);
					recog.evidenceDecl()?;

					}
				}

			EAL_T__27 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule assumptionDecl*/
					recog.base.set_state(185);
					recog.assumptionDecl()?;

					}
				}

			EAL_T__29 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule reasoningDecl*/
					recog.base.set_state(186);
					recog.reasoningDecl()?;

					}
				}

			EAL_T__36 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					/*InvokeRule claimDecl*/
					recog.base.set_state(187);
					recog.claimDecl()?;

					}
				}

			EAL_T__39 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					/*InvokeRule argumentDecl*/
					recog.base.set_state(188);
					recog.argumentDecl()?;

					}
				}

			EAL_T__56 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					/*InvokeRule objectionDecl*/
					recog.base.set_state(189);
					recog.objectionDecl()?;

					}
				}

			EAL_T__48 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					/*InvokeRule patternDecl*/
					recog.base.set_state(190);
					recog.patternDecl()?;

					}
				}

			EAL_T__53 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					/*InvokeRule applicationDecl*/
					recog.base.set_state(191);
					recog.applicationDecl()?;

					}
				}

			EAL_T__54 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					/*InvokeRule patternGuard*/
					recog.base.set_state(192);
					recog.patternGuard()?;

					}
				}

			EAL_T__58 |EAL_T__59 |EAL_T__60 |EAL_T__61 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					/*InvokeRule argumentationDirective*/
					recog.base.set_state(193);
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
			recog.base.set_state(196);
			recog.base.match_token(EAL_T__1,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(197);
			recog.qualifiedName()?;

			recog.base.set_state(198);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(202);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(199);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(204);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(214);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(205);
				recog.declaration()?;

				recog.base.set_state(209);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(206);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(211);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(216);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(217);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(219);
			recog.base.match_token(EAL_T__4,&mut recog.err_handler)?;

			recog.base.set_state(220);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			recog.base.set_state(221);
			recog.base.match_token(EAL_T__5,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(222);
			recog.identifier()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(223);
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
			recog.base.set_state(225);
			recog.base.match_token(EAL_T__6,&mut recog.err_handler)?;

			/*InvokeRule contextAttribute*/
			recog.base.set_state(226);
			recog.contextAttribute()?;

			recog.base.set_state(231);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__7 {
				{
				{
				recog.base.set_state(227);
				recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

				/*InvokeRule contextAttribute*/
				recog.base.set_state(228);
				recog.contextAttribute()?;

				}
				}
				recog.base.set_state(233);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(234);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(238);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(235);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(240);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(250);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(241);
				recog.declaration()?;

				recog.base.set_state(245);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(242);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(247);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(252);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(253);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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

			recog.base.set_state(285);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(255);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(256);
					recog.qualifiedName()?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(257);
					recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(258);
					recog.qualifiedName()?;

					}
				}

			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(259);
					recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

					/*InvokeRule signedNumber*/
					recog.base.set_state(260);
					recog.signedNumber()?;

					}
				}

			EAL_T__11 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(261);
					recog.base.match_token(EAL_T__11,&mut recog.err_handler)?;

					/*InvokeRule identifier*/
					recog.base.set_state(262);
					recog.identifier()?;

					}
				}

			EAL_T__12 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(263);
					recog.base.match_token(EAL_T__12,&mut recog.err_handler)?;

					/*InvokeRule jsonValue*/
					recog.base.set_state(264);
					recog.jsonValue()?;

					}
				}

			EAL_T__13 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(265);
					recog.base.match_token(EAL_T__13,&mut recog.err_handler)?;

					recog.base.set_state(266);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__14 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					recog.base.set_state(267);
					recog.base.match_token(EAL_T__14,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(268);
					recog.qualifiedName()?;

					}
				}

			EAL_T__15 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					recog.base.set_state(269);
					recog.base.match_token(EAL_T__15,&mut recog.err_handler)?;

					recog.base.set_state(270);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__16 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					recog.base.set_state(271);
					recog.base.match_token(EAL_T__16,&mut recog.err_handler)?;

					recog.base.set_state(272);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__17 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					recog.base.set_state(273);
					recog.base.match_token(EAL_T__17,&mut recog.err_handler)?;

					recog.base.set_state(274);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__18 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					recog.base.set_state(275);
					recog.base.match_token(EAL_T__18,&mut recog.err_handler)?;

					recog.base.set_state(276);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__19 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					recog.base.set_state(277);
					recog.base.match_token(EAL_T__19,&mut recog.err_handler)?;

					recog.base.set_state(278);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__20 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					recog.base.set_state(279);
					recog.base.match_token(EAL_T__20,&mut recog.err_handler)?;

					recog.base.set_state(280);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					recog.base.set_state(281);
					recog.base.match_token(EAL_T__21,&mut recog.err_handler)?;

					recog.base.set_state(282);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__22 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					recog.base.set_state(283);
					recog.base.match_token(EAL_T__22,&mut recog.err_handler)?;

					/*InvokeRule jsonValue*/
					recog.base.set_state(284);
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
			recog.base.set_state(287);
			recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(288);
			recog.qualifiedName()?;

			recog.base.set_state(289);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(293);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(290);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(295);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(297); 
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			loop {
				{
				{
				/*InvokeRule predicate*/
				recog.base.set_state(296);
				recog.predicate()?;

				}
				}
				recog.base.set_state(299); 
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				if !(_la==EAL_T__66) {break}
			}
			recog.base.set_state(301);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(303);
			recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(304);
			recog.qualifiedName()?;

			recog.base.set_state(305);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(309);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(306);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(311);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(315);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__13 {
				{
				{
				/*InvokeRule versionField*/
				recog.base.set_state(312);
				recog.versionField()?;

				}
				}
				recog.base.set_state(317);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(318);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(320);
			recog.base.match_token(EAL_T__13,&mut recog.err_handler)?;

			recog.base.set_state(321);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(322);
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
			recog.base.set_state(324);
			recog.base.match_token(EAL_T__23,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(325);
			recog.qualifiedName()?;

			recog.base.set_state(326);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(330);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(327);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(332);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(336);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 167788032) != 0) || _la==EAL_T__66 {
				{
				{
				/*InvokeRule evidenceField*/
				recog.base.set_state(333);
				recog.evidenceField()?;

				}
				}
				recog.base.set_state(338);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(339);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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

			recog.base.set_state(349);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule toolField*/
					recog.base.set_state(341);
					recog.toolField()?;

					}
				}

			EAL_T__26 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule deriveField*/
					recog.base.set_state(342);
					recog.deriveField()?;

					}
				}

			EAL_T__11 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule kindField*/
					recog.base.set_state(343);
					recog.kindField()?;

					}
				}

			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(344);
					recog.environmentField()?;

					}
				}

			EAL_T__10 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule maxAgeField*/
					recog.base.set_state(345);
					recog.maxAgeField()?;

					}
				}

			EAL_T__12 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule inputField*/
					recog.base.set_state(346);
					recog.inputField()?;

					}
				}

			EAL_T__24 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule inputBindingField*/
					recog.base.set_state(347);
					recog.inputBindingField()?;

					}
				}

			EAL_T__66 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule predicate*/
					recog.base.set_state(348);
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
			recog.base.set_state(351);
			recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(352);
			recog.qualifiedName()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(353);
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
			recog.base.set_state(355);
			recog.base.match_token(EAL_T__11,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(356);
			recog.identifier()?;

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
			recog.base.set_state(359);
			recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(360);
			recog.qualifiedName()?;

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
			recog.base.set_state(363);
			recog.base.match_token(EAL_T__10,&mut recog.err_handler)?;

			/*InvokeRule signedNumber*/
			recog.base.set_state(364);
			recog.signedNumber()?;

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
			recog.base.set_state(367);
			recog.base.match_token(EAL_T__12,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(368);
			recog.jsonValue()?;

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
			recog.base.set_state(371);
			recog.base.match_token(EAL_T__24,&mut recog.err_handler)?;

			recog.base.set_state(372);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			recog.base.set_state(373);
			recog.base.match_token(EAL_T__25,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(374);
			recog.qualifiedName()?;

			recog.base.set_state(375);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(376);
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
			recog.base.set_state(378);
			recog.base.match_token(EAL_T__26,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(379);
			recog.expression()?;

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
			recog.base.set_state(382);
			recog.base.match_token(EAL_T__27,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(383);
			recog.qualifiedName()?;

			recog.base.set_state(384);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(388);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(385);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(390);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(394);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 543195648) != 0) {
				{
				{
				/*InvokeRule assumptionField*/
				recog.base.set_state(391);
				recog.assumptionField()?;

				}
				}
				recog.base.set_state(396);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(397);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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

			recog.base.set_state(404);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__28 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule statementField*/
					recog.base.set_state(399);
					recog.statementField()?;

					}
				}

			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(400);
					recog.environmentField()?;

					}
				}

			EAL_T__14 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule validateField*/
					recog.base.set_state(401);
					recog.validateField()?;

					}
				}

			EAL_T__20 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule validFromField*/
					recog.base.set_state(402);
					recog.validFromField()?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule validUntilField*/
					recog.base.set_state(403);
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
			recog.base.set_state(406);
			recog.base.match_token(EAL_T__28,&mut recog.err_handler)?;

			recog.base.set_state(407);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(408);
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
			recog.base.set_state(410);
			recog.base.match_token(EAL_T__14,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(411);
			recog.qualifiedName()?;

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
			recog.base.set_state(414);
			recog.base.match_token(EAL_T__20,&mut recog.err_handler)?;

			recog.base.set_state(415);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

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
			recog.base.set_state(422);
			recog.base.match_token(EAL_T__29,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(423);
			recog.qualifiedName()?;

			recog.base.set_state(424);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(428);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(425);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(430);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(434);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 16)) & !0x3f) == 0 && ((1usize << (_la - 16)) & 229377) != 0) || _la==EAL_T__66 {
				{
				{
				/*InvokeRule reasoningField*/
				recog.base.set_state(431);
				recog.reasoningField()?;

				}
				}
				recog.base.set_state(436);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(437);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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

			recog.base.set_state(444);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__15 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule methodField*/
					recog.base.set_state(439);
					recog.methodField()?;

					}
				}

			EAL_T__30 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule rationaleField*/
					recog.base.set_state(440);
					recog.rationaleField()?;

					}
				}

			EAL_T__31 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule backingField*/
					recog.base.set_state(441);
					recog.backingField()?;

					}
				}

			EAL_T__32 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule transferField*/
					recog.base.set_state(442);
					recog.transferField()?;

					}
				}

			EAL_T__66 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule predicate*/
					recog.base.set_state(443);
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
			recog.base.set_state(446);
			recog.base.match_token(EAL_T__15,&mut recog.err_handler)?;

			recog.base.set_state(447);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(448);
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
			recog.base.set_state(450);
			recog.base.match_token(EAL_T__30,&mut recog.err_handler)?;

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
			recog.base.set_state(454);
			recog.base.match_token(EAL_T__31,&mut recog.err_handler)?;

			/*InvokeRule referenceList*/
			recog.base.set_state(455);
			recog.referenceList()?;

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
			recog.base.set_state(458);
			recog.base.match_token(EAL_T__32,&mut recog.err_handler)?;

			recog.base.set_state(459);
			recog.base.match_token(EAL_T__25,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(460);
			recog.qualifiedName()?;

			recog.base.set_state(461);
			recog.base.match_token(EAL_T__33,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(462);
			recog.qualifiedName()?;

			recog.base.set_state(463);
			recog.base.match_token(EAL_T__34,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(464);
			recog.qualifiedName()?;

			recog.base.set_state(465);
			recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

			recog.base.set_state(466);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(467);
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
			recog.base.set_state(469);
			recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(470);
			recog.qualifiedName()?;

			recog.base.set_state(471);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(475);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(472);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(477);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(481);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 9)) & !0x3f) == 0 && ((1usize << (_la - 9)) & 537919489) != 0) {
				{
				{
				/*InvokeRule claimField*/
				recog.base.set_state(478);
				recog.claimField()?;

				}
				}
				recog.base.set_state(483);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(484);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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

			recog.base.set_state(489);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__28 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule statementField*/
					recog.base.set_state(486);
					recog.statementField()?;

					}
				}

			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule environmentField*/
					recog.base.set_state(487);
					recog.environmentField()?;

					}
				}

			EAL_T__37 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule propositionDecl*/
					recog.base.set_state(488);
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
			recog.base.set_state(491);
			recog.base.match_token(EAL_T__37,&mut recog.err_handler)?;

			recog.base.set_state(492);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(496);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(493);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(498);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(502);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 17)) & !0x3f) == 0 && ((1usize << (_la - 17)) & 4194431) != 0) {
				{
				{
				/*InvokeRule propositionField*/
				recog.base.set_state(499);
				recog.propositionField()?;

				}
				}
				recog.base.set_state(504);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(505);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(509);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(506);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(511);
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

			recog.base.set_state(520);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__16 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule subjectField*/
					recog.base.set_state(512);
					recog.subjectField()?;

					}
				}

			EAL_T__17 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule quantityField*/
					recog.base.set_state(513);
					recog.quantityField()?;

					}
				}

			EAL_T__18 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule unitField*/
					recog.base.set_state(514);
					recog.unitField()?;

					}
				}

			EAL_T__19 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule scopeField*/
					recog.base.set_state(515);
					recog.scopeField()?;

					}
				}

			EAL_T__20 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule validFromField*/
					recog.base.set_state(516);
					recog.validFromField()?;

					}
				}

			EAL_T__21 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					/*InvokeRule validUntilField*/
					recog.base.set_state(517);
					recog.validUntilField()?;

					}
				}

			EAL_T__22 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule queryField*/
					recog.base.set_state(518);
					recog.queryField()?;

					}
				}

			EAL_T__38 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					/*InvokeRule resultField*/
					recog.base.set_state(519);
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
			recog.base.set_state(522);
			recog.base.match_token(EAL_T__16,&mut recog.err_handler)?;

			recog.base.set_state(523);
			recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(524);
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
			recog.base.set_state(538);
			recog.base.match_token(EAL_T__22,&mut recog.err_handler)?;

			/*InvokeRule jsonValue*/
			recog.base.set_state(539);
			recog.jsonValue()?;

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
			recog.base.set_state(542);
			recog.base.match_token(EAL_T__38,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(543);
			recog.expression()?;

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
			recog.base.set_state(546);
			recog.base.match_token(EAL_T__39,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(547);
			recog.qualifiedName()?;

			recog.base.set_state(581);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__40 
				=> {
					{
					recog.base.set_state(548);
					recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

					recog.base.set_state(552);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(549);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(554);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(555);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(556);
					recog.lineEnd()?;

					}
				}

			EAL_T__2 
				=> {
					{
					recog.base.set_state(558);
					recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

					recog.base.set_state(562);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(559);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(564);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(574);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(565);
						recog.declaration()?;

						recog.base.set_state(569);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(566);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(571);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(576);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(577);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(578);
					recog.lineEnd()?;

					recog.base.set_state(579);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(583);
			recog.support()?;

			recog.base.set_state(587);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(584);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(589);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(590);
			recog.base.match_token(EAL_T__41,&mut recog.err_handler)?;

			recog.base.set_state(594);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(591);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(596);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(597);
			let tmp = recog.reference()?;
			// SAFETY: the borrow only covers this one field access, during which no other handle to the context is dereferenced.
			(unsafe { cast_mut::<ArgumentFlowContext >(&mut _localctx) }).reasoningRef = Some(tmp.clone());
			  

			recog.base.set_state(601);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(598);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(603);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(604);
			recog.base.match_token(EAL_T__42,&mut recog.err_handler)?;

			recog.base.set_state(608);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(605);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(610);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(611);
			let tmp = recog.reference()?;
			// SAFETY: the borrow only covers this one field access, during which no other handle to the context is dereferenced.
			(unsafe { cast_mut::<ArgumentFlowContext >(&mut _localctx) }).conclusionRef = Some(tmp.clone());
			  

			recog.base.set_state(620);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(42,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(615);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(612);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(617);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(618);
					recog.base.match_token(EAL_T__43,&mut recog.err_handler)?;

					/*InvokeRule reference*/
					recog.base.set_state(619);
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
			recog.base.set_state(622);
			recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

			recog.base.set_state(626);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(43,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(623);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(628);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(43,&mut recog.base)?;
			}
			recog.base.set_state(643);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if ((((_la - 24)) & !0x3f) == 0 && ((1usize << (_la - 24)) & 25165825) != 0) {
				{
				/*InvokeRule supportGroup*/
				recog.base.set_state(629);
				recog.supportGroup()?;

				recog.base.set_state(640);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__7 {
					{
					{
					recog.base.set_state(630);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(634);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(631);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(636);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule supportGroup*/
					recog.base.set_state(637);
					recog.supportGroup()?;

					}
					}
					recog.base.set_state(642);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(648);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(645);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(650);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(651);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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
			recog.base.set_state(653);
			recog.supportKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(654);
			recog.reference()?;

			recog.base.set_state(665);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(49,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(655);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(659);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(656);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(661);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule reference*/
					recog.base.set_state(662);
					recog.reference()?;

					}
					} 
				}
				recog.base.set_state(667);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(49,&mut recog.base)?;
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
			recog.base.set_state(668);
			_la = recog.base.input.la(1);
			if { !(((((_la - 24)) & !0x3f) == 0 && ((1usize << (_la - 24)) & 25165825) != 0)) } {
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
			recog.base.set_state(670);
			recog.base.match_token(EAL_T__48,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(671);
			recog.qualifiedName()?;

			recog.base.set_state(672);
			recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

			recog.base.set_state(676);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(50,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(673);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(678);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(50,&mut recog.base)?;
			}
			recog.base.set_state(693);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 245301476) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4243130991) != 0) || _la==EAL_T__64 || _la==EAL_ID {
				{
				/*InvokeRule patternParameter*/
				recog.base.set_state(679);
				recog.patternParameter()?;

				recog.base.set_state(690);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__7 {
					{
					{
					recog.base.set_state(680);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(684);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(681);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(686);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule patternParameter*/
					recog.base.set_state(687);
					recog.patternParameter()?;

					}
					}
					recog.base.set_state(692);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(698);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(695);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(700);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(701);
			recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

			recog.base.set_state(710);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(56,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(705);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(702);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(707);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(708);
					recog.base.match_token(EAL_T__51,&mut recog.err_handler)?;

					/*InvokeRule identifier*/
					recog.base.set_state(709);
					recog.identifier()?;

					}
				}

				_ => {}
			}
			recog.base.set_state(715);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(712);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(717);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(748);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__40 
				=> {
					{
					recog.base.set_state(718);
					recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

					recog.base.set_state(722);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(719);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(724);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule argumentFlow*/
					recog.base.set_state(725);
					recog.argumentFlow()?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(726);
					recog.lineEnd()?;

					}
				}

			EAL_T__2 
				=> {
					{
					recog.base.set_state(728);
					recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

					recog.base.set_state(732);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(729);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(734);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(742); 
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					loop {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(735);
						recog.declaration()?;

						recog.base.set_state(739);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(736);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(741);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(744); 
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						if !((((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0)) {break}
					}
					recog.base.set_state(746);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(750);
			recog.identifier()?;

			recog.base.set_state(751);
			recog.base.match_token(EAL_T__52,&mut recog.err_handler)?;

			/*InvokeRule parameterKind*/
			recog.base.set_state(752);
			recog.parameterKind()?;

			recog.base.set_state(755);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__44 {
				{
				recog.base.set_state(753);
				recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

				recog.base.set_state(754);
				recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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

			recog.base.set_state(764);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__36 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(757);
					recog.base.match_token(EAL_T__36,&mut recog.err_handler)?;

					}
				}

			EAL_T__29 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(758);
					recog.base.match_token(EAL_T__29,&mut recog.err_handler)?;

					}
				}

			EAL_T__23 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(759);
					recog.base.match_token(EAL_T__23,&mut recog.err_handler)?;

					}
				}

			EAL_T__27 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(760);
					recog.base.match_token(EAL_T__27,&mut recog.err_handler)?;

					}
				}

			EAL_T__8 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(761);
					recog.base.match_token(EAL_T__8,&mut recog.err_handler)?;

					}
				}

			EAL_T__9 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(762);
					recog.base.match_token(EAL_T__9,&mut recog.err_handler)?;

					}
				}

			EAL_T__1 |EAL_T__4 |EAL_T__5 |EAL_T__6 |EAL_T__15 |EAL_T__16 |EAL_T__17 |
			EAL_T__18 |EAL_T__19 |EAL_T__22 |EAL_T__24 |EAL_T__25 |EAL_T__26 |EAL_T__32 |
			EAL_T__33 |EAL_T__34 |EAL_T__35 |EAL_T__37 |EAL_T__38 |EAL_T__41 |EAL_T__43 |
			EAL_T__48 |EAL_T__51 |EAL_T__53 |EAL_T__54 |EAL_T__55 |EAL_T__58 |EAL_T__59 |
			EAL_T__60 |EAL_T__61 |EAL_T__62 |EAL_T__63 |EAL_T__64 |EAL_ID 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					/*InvokeRule identifier*/
					recog.base.set_state(763);
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
			recog.base.set_state(766);
			recog.base.match_token(EAL_T__53,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(767);
			recog.qualifiedName()?;

			recog.base.set_state(768);
			recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(769);
			recog.qualifiedName()?;

			recog.base.set_state(770);
			recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

			recog.base.set_state(774);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(65,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(771);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(776);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(65,&mut recog.base)?;
			}
			recog.base.set_state(791);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 245301476) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4243130991) != 0) || _la==EAL_T__64 || _la==EAL_ID {
				{
				/*InvokeRule patternBinding*/
				recog.base.set_state(777);
				recog.patternBinding()?;

				recog.base.set_state(788);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__7 {
					{
					{
					recog.base.set_state(778);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(782);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(779);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(784);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule patternBinding*/
					recog.base.set_state(785);
					recog.patternBinding()?;

					}
					}
					recog.base.set_state(790);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(796);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(793);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(798);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(799);
			recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(800);
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
			recog.base.set_state(802);
			recog.identifier()?;

			recog.base.set_state(803);
			recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

			/*InvokeRule bindingValue*/
			recog.base.set_state(804);
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
			recog.base.set_state(837);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__1 |EAL_T__4 |EAL_T__5 |EAL_T__6 |EAL_T__15 |EAL_T__16 |EAL_T__17 |
			EAL_T__18 |EAL_T__19 |EAL_T__22 |EAL_T__24 |EAL_T__25 |EAL_T__26 |EAL_T__32 |
			EAL_T__33 |EAL_T__34 |EAL_T__35 |EAL_T__37 |EAL_T__38 |EAL_T__41 |EAL_T__43 |
			EAL_T__48 |EAL_T__51 |EAL_T__53 |EAL_T__54 |EAL_T__55 |EAL_T__58 |EAL_T__59 |
			EAL_T__60 |EAL_T__61 |EAL_T__62 |EAL_T__63 |EAL_T__64 |EAL_ID 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule reference*/
					recog.base.set_state(806);
					recog.reference()?;

					}
				}

			EAL_T__44 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(807);
					recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

					recog.base.set_state(811);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(70,&mut recog.base)?;
					while { _alt!=2 && _alt!=INVALID_ALT } {
						if _alt==1 {
							{
							{
							recog.base.set_state(808);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							} 
						}
						recog.base.set_state(813);
						recog.err_handler.sync(&mut recog.base)?;
						_alt = recog.interpreter.adaptive_predict(70,&mut recog.base)?;
					}
					recog.base.set_state(828);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if (((_la) & !0x3f) == 0 && ((1usize << _la) & 245301476) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4243130991) != 0) || _la==EAL_T__64 || _la==EAL_ID {
						{
						/*InvokeRule reference*/
						recog.base.set_state(814);
						recog.reference()?;

						recog.base.set_state(825);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_T__7 {
							{
							{
							recog.base.set_state(815);
							recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

							recog.base.set_state(819);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
							while _la==EAL_NL {
								{
								{
								recog.base.set_state(816);
								recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

								}
								}
								recog.base.set_state(821);
								recog.err_handler.sync(&mut recog.base)?;
								_la = recog.base.input.la(1);
							}
							/*InvokeRule reference*/
							recog.base.set_state(822);
							recog.reference()?;

							}
							}
							recog.base.set_state(827);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
					}

					recog.base.set_state(833);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(830);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(835);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(836);
					recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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
			recog.base.set_state(839);
			recog.base.match_token(EAL_T__54,&mut recog.err_handler)?;

			/*InvokeRule identifier*/
			recog.base.set_state(840);
			recog.identifier()?;

			recog.base.set_state(841);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(845);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(842);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(847);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(857);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
				{
				{
				/*InvokeRule declaration*/
				recog.base.set_state(848);
				recog.declaration()?;

				recog.base.set_state(852);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(849);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(854);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
				}
				recog.base.set_state(859);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(860);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

			recog.base.set_state(888);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(83,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(864);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(861);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(866);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(867);
					recog.base.match_token(EAL_T__55,&mut recog.err_handler)?;

					recog.base.set_state(868);
					recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

					recog.base.set_state(872);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(869);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(874);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(884);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while (((_la) & !0x3f) == 0 && ((1usize << _la) & 1358956196) != 0) || ((((_la - 37)) & !0x3f) == 0 && ((1usize << (_la - 37)) & 64360457) != 0) {
						{
						{
						/*InvokeRule declaration*/
						recog.base.set_state(875);
						recog.declaration()?;

						recog.base.set_state(879);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(876);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(881);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
						}
						recog.base.set_state(886);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(887);
					recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(890);
			recog.base.match_token(EAL_T__56,&mut recog.err_handler)?;

			/*InvokeRule qualifiedName*/
			recog.base.set_state(891);
			recog.qualifiedName()?;

			recog.base.set_state(892);
			recog.base.match_token(EAL_T__40,&mut recog.err_handler)?;

			recog.base.set_state(896);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(893);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(898);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule objectionSupport*/
			recog.base.set_state(899);
			recog.objectionSupport()?;

			recog.base.set_state(903);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(900);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(905);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(906);
			recog.base.match_token(EAL_T__57,&mut recog.err_handler)?;

			recog.base.set_state(910);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(907);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(912);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule targetKind*/
			recog.base.set_state(913);
			recog.targetKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(914);
			recog.reference()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(915);
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
			recog.base.set_state(917);
			recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

			recog.base.set_state(921);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(87,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(918);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(923);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(87,&mut recog.base)?;
			}
			recog.base.set_state(938);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__23 || _la==EAL_T__47 {
				{
				/*InvokeRule objectionGroup*/
				recog.base.set_state(924);
				recog.objectionGroup()?;

				recog.base.set_state(935);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_T__7 {
					{
					{
					recog.base.set_state(925);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(929);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(926);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(931);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule objectionGroup*/
					recog.base.set_state(932);
					recog.objectionGroup()?;

					}
					}
					recog.base.set_state(937);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				}
			}

			recog.base.set_state(943);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(940);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(945);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(946);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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
			recog.base.set_state(948);
			recog.objectionSupportKind()?;

			/*InvokeRule reference*/
			recog.base.set_state(949);
			recog.reference()?;

			recog.base.set_state(960);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(93,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(950);
					recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

					recog.base.set_state(954);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==EAL_NL {
						{
						{
						recog.base.set_state(951);
						recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(956);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule reference*/
					recog.base.set_state(957);
					recog.reference()?;

					}
					} 
				}
				recog.base.set_state(962);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(93,&mut recog.base)?;
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
			recog.base.set_state(963);
			_la = recog.base.input.la(1);
			if { !(_la==EAL_T__23 || _la==EAL_T__47) } {
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
			recog.base.set_state(965);
			_la = recog.base.input.la(1);
			if { !(((((_la - 28)) & !0x3f) == 0 && ((1usize << (_la - 28)) & 536875525) != 0)) } {
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

			recog.base.set_state(996);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__58 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(967);
					recog.base.match_token(EAL_T__58,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(968);
					recog.qualifiedName()?;

					recog.base.set_state(969);
					recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

					recog.base.set_state(970);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(971);
					recog.lineEnd()?;

					}
				}

			EAL_T__59 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(973);
					recog.base.match_token(EAL_T__59,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(974);
					recog.qualifiedName()?;

					/*InvokeRule signedNumber*/
					recog.base.set_state(975);
					recog.signedNumber()?;

					recog.base.set_state(976);
					recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

					recog.base.set_state(977);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(978);
					recog.lineEnd()?;

					}
				}

			EAL_T__60 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(980);
					recog.base.match_token(EAL_T__60,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(981);
					recog.qualifiedName()?;

					recog.base.set_state(982);
					recog.base.match_token(EAL_T__33,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(983);
					recog.qualifiedName()?;

					recog.base.set_state(984);
					recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

					recog.base.set_state(985);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(986);
					recog.lineEnd()?;

					}
				}

			EAL_T__61 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(988);
					recog.base.match_token(EAL_T__61,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(989);
					recog.qualifiedName()?;

					recog.base.set_state(990);
					recog.base.match_token(EAL_T__62,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(991);
					recog.qualifiedName()?;

					recog.base.set_state(992);
					recog.base.match_token(EAL_T__35,&mut recog.err_handler)?;

					recog.base.set_state(993);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					/*InvokeRule lineEnd*/
					recog.base.set_state(994);
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
			recog.base.set_state(998);
			recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

			recog.base.set_state(1002);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(999);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1004);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule reference*/
			recog.base.set_state(1005);
			recog.reference()?;

			recog.base.set_state(1016);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__7 {
				{
				{
				recog.base.set_state(1006);
				recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

				recog.base.set_state(1010);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1007);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1012);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				/*InvokeRule reference*/
				recog.base.set_state(1013);
				recog.reference()?;

				}
				}
				recog.base.set_state(1018);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1022);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1019);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1024);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1025);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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

			recog.base.set_state(1038);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(99,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1027);
					recog.qualifiedName()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(1028);
					recog.base.match_token(EAL_T__63,&mut recog.err_handler)?;

					recog.base.set_state(1029);
					recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(1030);
					recog.qualifiedName()?;

					recog.base.set_state(1031);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(1033);
					recog.base.match_token(EAL_T__64,&mut recog.err_handler)?;

					recog.base.set_state(1034);
					recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

					/*InvokeRule qualifiedName*/
					recog.base.set_state(1035);
					recog.qualifiedName()?;

					recog.base.set_state(1036);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

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
			recog.base.set_state(1040);
			recog.identifier()?;

			recog.base.set_state(1045);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__65 {
				{
				{
				recog.base.set_state(1041);
				recog.base.match_token(EAL_T__65,&mut recog.err_handler)?;

				/*InvokeRule identifier*/
				recog.base.set_state(1042);
				recog.identifier()?;

				}
				}
				recog.base.set_state(1047);
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
			recog.base.set_state(1048);
			recog.base.match_token(EAL_T__66,&mut recog.err_handler)?;

			/*InvokeRule expression*/
			recog.base.set_state(1049);
			recog.expression()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(1050);
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
			recog.base.set_state(1052);
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
			recog.base.set_state(1054);
			recog.andExpression()?;

			recog.base.set_state(1059);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__67 {
				{
				{
				recog.base.set_state(1055);
				recog.base.match_token(EAL_T__67,&mut recog.err_handler)?;

				/*InvokeRule andExpression*/
				recog.base.set_state(1056);
				recog.andExpression()?;

				}
				}
				recog.base.set_state(1061);
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
			recog.base.set_state(1062);
			recog.comparisonExpression()?;

			recog.base.set_state(1067);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__68 {
				{
				{
				recog.base.set_state(1063);
				recog.base.match_token(EAL_T__68,&mut recog.err_handler)?;

				/*InvokeRule comparisonExpression*/
				recog.base.set_state(1064);
				recog.comparisonExpression()?;

				}
				}
				recog.base.set_state(1069);
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
			recog.base.set_state(1070);
			recog.additiveExpression()?;

			recog.base.set_state(1074);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if ((((_la - 76)) & !0x3f) == 0 && ((1usize << (_la - 76)) & 63) != 0) {
				{
				/*InvokeRule comparator*/
				recog.base.set_state(1071);
				recog.comparator()?;

				/*InvokeRule additiveExpression*/
				recog.base.set_state(1072);
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
			recog.base.set_state(1076);
			recog.multiplicativeExpression()?;

			recog.base.set_state(1081);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_T__69 || _la==EAL_T__70 {
				{
				{
				recog.base.set_state(1077);
				_la = recog.base.input.la(1);
				if { !(_la==EAL_T__69 || _la==EAL_T__70) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				/*InvokeRule multiplicativeExpression*/
				recog.base.set_state(1078);
				recog.multiplicativeExpression()?;

				}
				}
				recog.base.set_state(1083);
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
			recog.base.set_state(1084);
			recog.unaryExpression()?;

			recog.base.set_state(1089);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while ((((_la - 72)) & !0x3f) == 0 && ((1usize << (_la - 72)) & 7) != 0) {
				{
				{
				recog.base.set_state(1085);
				_la = recog.base.input.la(1);
				if { !(((((_la - 72)) & !0x3f) == 0 && ((1usize << (_la - 72)) & 7) != 0)) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				/*InvokeRule unaryExpression*/
				recog.base.set_state(1086);
				recog.unaryExpression()?;

				}
				}
				recog.base.set_state(1091);
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

			recog.base.set_state(1095);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(106,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1092);
					_la = recog.base.input.la(1);
					if { !(((((_la - 70)) & !0x3f) == 0 && ((1usize << (_la - 70)) & 35) != 0)) } {
						recog.err_handler.recover_inline(&mut recog.base)?;

					}
					else {
						if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
						recog.err_handler.report_match(&mut recog.base);
						recog.base.consume(&mut recog.err_handler);
					}
					/*InvokeRule unaryExpression*/
					recog.base.set_state(1093);
					recog.unaryExpression()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule primaryExpression*/
					recog.base.set_state(1094);
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

			recog.base.set_state(1117);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(109,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1097);
					recog.qualifiedName()?;

					recog.base.set_state(1098);
					recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

					recog.base.set_state(1107);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					if (((_la) & !0x3f) == 0 && ((1usize << _la) & 245301484) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4243266159) != 0) || ((((_la - 65)) & !0x3f) == 0 && ((1usize << (_la - 65)) & 8258657) != 0) {
						{
						/*InvokeRule expression*/
						recog.base.set_state(1099);
						recog.expression()?;

						recog.base.set_state(1104);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_T__7 {
							{
							{
							recog.base.set_state(1100);
							recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

							/*InvokeRule expression*/
							recog.base.set_state(1101);
							recog.expression()?;

							}
							}
							recog.base.set_state(1106);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						}
					}

					recog.base.set_state(1109);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule qualifiedName*/
					recog.base.set_state(1111);
					recog.qualifiedName()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule jsonValue*/
					recog.base.set_state(1112);
					recog.jsonValue()?;

					}
				}
			,
				4 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(1113);
					recog.base.match_token(EAL_T__49,&mut recog.err_handler)?;

					/*InvokeRule expression*/
					recog.base.set_state(1114);
					recog.expression()?;

					recog.base.set_state(1115);
					recog.base.match_token(EAL_T__50,&mut recog.err_handler)?;

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
			recog.base.set_state(1119);
			_la = recog.base.input.la(1);
			if { !(((((_la - 76)) & !0x3f) == 0 && ((1usize << (_la - 76)) & 63) != 0)) } {
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

			recog.base.set_state(1124);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_T__70 |EAL_T__81 |EAL_T__82 |EAL_T__83 |EAL_NUMBER |EAL_STRING 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule jsonScalar*/
					recog.base.set_state(1121);
					recog.jsonScalar()?;

					}
				}

			EAL_T__2 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule jsonObject*/
					recog.base.set_state(1122);
					recog.jsonObject()?;

					}
				}

			EAL_T__44 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule jsonArray*/
					recog.base.set_state(1123);
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
			recog.base.set_state(1126);
			recog.base.match_token(EAL_T__2,&mut recog.err_handler)?;

			recog.base.set_state(1130);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(111,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(1127);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(1132);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(111,&mut recog.base)?;
			}
			recog.base.set_state(1181);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_STRING {
				{
				recog.base.set_state(1133);
				recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

				recog.base.set_state(1137);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1134);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1139);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				recog.base.set_state(1140);
				recog.base.match_token(EAL_T__52,&mut recog.err_handler)?;

				recog.base.set_state(1144);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				while _la==EAL_NL {
					{
					{
					recog.base.set_state(1141);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					}
					recog.base.set_state(1146);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
				}
				/*InvokeRule jsonValue*/
				recog.base.set_state(1147);
				recog.jsonValue()?;

				recog.base.set_state(1178);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(118,&mut recog.base)?;
				while { _alt!=2 && _alt!=INVALID_ALT } {
					if _alt==1 {
						{
						{
						recog.base.set_state(1151);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1148);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1153);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1154);
						recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

						recog.base.set_state(1158);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1155);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1160);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1161);
						recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

						recog.base.set_state(1165);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1162);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1167);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1168);
						recog.base.match_token(EAL_T__52,&mut recog.err_handler)?;

						recog.base.set_state(1172);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1169);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1174);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						/*InvokeRule jsonValue*/
						recog.base.set_state(1175);
						recog.jsonValue()?;

						}
						} 
					}
					recog.base.set_state(1180);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(118,&mut recog.base)?;
				}
				}
			}

			recog.base.set_state(1186);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1183);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1188);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1189);
			recog.base.match_token(EAL_T__3,&mut recog.err_handler)?;

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
			recog.base.set_state(1191);
			recog.base.match_token(EAL_T__44,&mut recog.err_handler)?;

			recog.base.set_state(1195);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(121,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(1192);
					recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(1197);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(121,&mut recog.base)?;
			}
			recog.base.set_state(1218);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__2 || _la==EAL_T__44 || _la==EAL_T__70 || ((((_la - 82)) & !0x3f) == 0 && ((1usize << (_la - 82)) & 55) != 0) {
				{
				/*InvokeRule jsonValue*/
				recog.base.set_state(1198);
				recog.jsonValue()?;

				recog.base.set_state(1215);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(124,&mut recog.base)?;
				while { _alt!=2 && _alt!=INVALID_ALT } {
					if _alt==1 {
						{
						{
						recog.base.set_state(1202);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1199);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1204);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						recog.base.set_state(1205);
						recog.base.match_token(EAL_T__7,&mut recog.err_handler)?;

						recog.base.set_state(1209);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
						while _la==EAL_NL {
							{
							{
							recog.base.set_state(1206);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}
							recog.base.set_state(1211);
							recog.err_handler.sync(&mut recog.base)?;
							_la = recog.base.input.la(1);
						}
						/*InvokeRule jsonValue*/
						recog.base.set_state(1212);
						recog.jsonValue()?;

						}
						} 
					}
					recog.base.set_state(1217);
					recog.err_handler.sync(&mut recog.base)?;
					_alt = recog.interpreter.adaptive_predict(124,&mut recog.base)?;
				}
				}
			}

			recog.base.set_state(1223);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==EAL_NL {
				{
				{
				recog.base.set_state(1220);
				recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(1225);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(1226);
			recog.base.match_token(EAL_T__45,&mut recog.err_handler)?;

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

			recog.base.set_state(1233);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_STRING 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1228);
					recog.base.match_token(EAL_STRING,&mut recog.err_handler)?;

					}
				}

			EAL_T__70 |EAL_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule signedNumber*/
					recog.base.set_state(1229);
					recog.signedNumber()?;

					}
				}

			EAL_T__81 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(1230);
					recog.base.match_token(EAL_T__81,&mut recog.err_handler)?;

					}
				}

			EAL_T__82 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(1231);
					recog.base.match_token(EAL_T__82,&mut recog.err_handler)?;

					}
				}

			EAL_T__83 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(1232);
					recog.base.match_token(EAL_T__83,&mut recog.err_handler)?;

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
			recog.base.set_state(1236);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==EAL_T__70 {
				{
				recog.base.set_state(1235);
				recog.base.match_token(EAL_T__70,&mut recog.err_handler)?;

				}
			}

			recog.base.set_state(1238);
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
			recog.base.set_state(1240);
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 245301476) != 0) || ((((_la - 33)) & !0x3f) == 0 && ((1usize << (_la - 33)) & 4243130991) != 0) || _la==EAL_T__64 || _la==EAL_ID) } {
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
			recog.base.set_state(1248);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			EAL_NL 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(1243); 
					recog.err_handler.sync(&mut recog.base)?;
					_alt = 1;
					loop {
						match _alt {
						    x if x == 1=>
							{
							{
							recog.base.set_state(1242);
							recog.base.match_token(EAL_NL,&mut recog.err_handler)?;

							}
							}

						_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
						}
						recog.base.set_state(1245); 
						recog.err_handler.sync(&mut recog.base)?;
						_alt = recog.interpreter.adaptive_predict(129,&mut recog.base)?;
						if _alt==2 || _alt==INVALID_ALT { break }
					}
					}
				}

			EAL_EOF 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(1247);
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
		4, 1, 91, 1251, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 
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
		161, 9, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 5, 0, 168, 8, 0, 10, 0, 12, 0, 
		171, 9, 0, 5, 0, 173, 8, 0, 10, 0, 12, 0, 176, 9, 0, 1, 0, 1, 0, 1, 1, 
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
		1, 1, 1, 1, 3, 1, 195, 8, 1, 1, 2, 1, 2, 1, 2, 1, 2, 5, 2, 201, 8, 2, 
		10, 2, 12, 2, 204, 9, 2, 1, 2, 1, 2, 5, 2, 208, 8, 2, 10, 2, 12, 2, 211, 
		9, 2, 5, 2, 213, 8, 2, 10, 2, 12, 2, 216, 9, 2, 1, 2, 1, 2, 1, 3, 1, 3, 
		1, 3, 1, 3, 1, 3, 1, 3, 1, 4, 1, 4, 1, 4, 1, 4, 5, 4, 230, 8, 4, 10, 4, 
		12, 4, 233, 9, 4, 1, 4, 1, 4, 5, 4, 237, 8, 4, 10, 4, 12, 4, 240, 9, 4, 
		1, 4, 1, 4, 5, 4, 244, 8, 4, 10, 4, 12, 4, 247, 9, 4, 5, 4, 249, 8, 4, 
		10, 4, 12, 4, 252, 9, 4, 1, 4, 1, 4, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 
		5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 
		5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 5, 1, 
		5, 3, 5, 286, 8, 5, 1, 6, 1, 6, 1, 6, 1, 6, 5, 6, 292, 8, 6, 10, 6, 12, 
		6, 295, 9, 6, 1, 6, 4, 6, 298, 8, 6, 11, 6, 12, 6, 299, 1, 6, 1, 6, 1, 
		7, 1, 7, 1, 7, 1, 7, 5, 7, 308, 8, 7, 10, 7, 12, 7, 311, 9, 7, 1, 7, 5, 
		7, 314, 8, 7, 10, 7, 12, 7, 317, 9, 7, 1, 7, 1, 7, 1, 8, 1, 8, 1, 8, 1, 
		8, 1, 9, 1, 9, 1, 9, 1, 9, 5, 9, 329, 8, 9, 10, 9, 12, 9, 332, 9, 9, 1, 
		9, 5, 9, 335, 8, 9, 10, 9, 12, 9, 338, 9, 9, 1, 9, 1, 9, 1, 10, 1, 10, 
		1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 3, 10, 350, 8, 10, 1, 11, 1, 
		11, 1, 11, 1, 11, 1, 12, 1, 12, 1, 12, 1, 12, 1, 13, 1, 13, 1, 13, 1, 
		13, 1, 14, 1, 14, 1, 14, 1, 14, 1, 15, 1, 15, 1, 15, 1, 15, 1, 16, 1, 
		16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16, 1, 17, 1, 17, 1, 17, 1, 17, 1, 
		18, 1, 18, 1, 18, 1, 18, 5, 18, 387, 8, 18, 10, 18, 12, 18, 390, 9, 18, 
		1, 18, 5, 18, 393, 8, 18, 10, 18, 12, 18, 396, 9, 18, 1, 18, 1, 18, 1, 
		19, 1, 19, 1, 19, 1, 19, 1, 19, 3, 19, 405, 8, 19, 1, 20, 1, 20, 1, 20, 
		1, 20, 1, 21, 1, 21, 1, 21, 1, 21, 1, 22, 1, 22, 1, 22, 1, 22, 1, 23, 
		1, 23, 1, 23, 1, 23, 1, 24, 1, 24, 1, 24, 1, 24, 5, 24, 427, 8, 24, 10, 
		24, 12, 24, 430, 9, 24, 1, 24, 5, 24, 433, 8, 24, 10, 24, 12, 24, 436, 
		9, 24, 1, 24, 1, 24, 1, 25, 1, 25, 1, 25, 1, 25, 1, 25, 3, 25, 445, 8, 
		25, 1, 26, 1, 26, 1, 26, 1, 26, 1, 27, 1, 27, 1, 27, 1, 27, 1, 28, 1, 
		28, 1, 28, 1, 28, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 29, 1, 
		29, 1, 29, 1, 29, 1, 29, 1, 30, 1, 30, 1, 30, 1, 30, 5, 30, 474, 8, 30, 
		10, 30, 12, 30, 477, 9, 30, 1, 30, 5, 30, 480, 8, 30, 10, 30, 12, 30, 
		483, 9, 30, 1, 30, 1, 30, 1, 31, 1, 31, 1, 31, 3, 31, 490, 8, 31, 1, 32, 
		1, 32, 1, 32, 5, 32, 495, 8, 32, 10, 32, 12, 32, 498, 9, 32, 1, 32, 5, 
		32, 501, 8, 32, 10, 32, 12, 32, 504, 9, 32, 1, 32, 1, 32, 5, 32, 508, 
		8, 32, 10, 32, 12, 32, 511, 9, 32, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 
		1, 33, 1, 33, 1, 33, 3, 33, 521, 8, 33, 1, 34, 1, 34, 1, 34, 1, 34, 1, 
		35, 1, 35, 1, 35, 1, 35, 1, 36, 1, 36, 1, 36, 1, 36, 1, 37, 1, 37, 1, 
		37, 1, 37, 1, 38, 1, 38, 1, 38, 1, 38, 1, 39, 1, 39, 1, 39, 1, 39, 1, 
		40, 1, 40, 1, 40, 1, 40, 5, 40, 551, 8, 40, 10, 40, 12, 40, 554, 9, 40, 
		1, 40, 1, 40, 1, 40, 1, 40, 1, 40, 5, 40, 561, 8, 40, 10, 40, 12, 40, 
		564, 9, 40, 1, 40, 1, 40, 5, 40, 568, 8, 40, 10, 40, 12, 40, 571, 9, 40, 
		5, 40, 573, 8, 40, 10, 40, 12, 40, 576, 9, 40, 1, 40, 1, 40, 1, 40, 1, 
		40, 3, 40, 582, 8, 40, 1, 41, 1, 41, 5, 41, 586, 8, 41, 10, 41, 12, 41, 
		589, 9, 41, 1, 41, 1, 41, 5, 41, 593, 8, 41, 10, 41, 12, 41, 596, 9, 41, 
		1, 41, 1, 41, 5, 41, 600, 8, 41, 10, 41, 12, 41, 603, 9, 41, 1, 41, 1, 
		41, 5, 41, 607, 8, 41, 10, 41, 12, 41, 610, 9, 41, 1, 41, 1, 41, 5, 41, 
		614, 8, 41, 10, 41, 12, 41, 617, 9, 41, 1, 41, 1, 41, 3, 41, 621, 8, 41, 
		1, 42, 1, 42, 5, 42, 625, 8, 42, 10, 42, 12, 42, 628, 9, 42, 1, 42, 1, 
		42, 1, 42, 5, 42, 633, 8, 42, 10, 42, 12, 42, 636, 9, 42, 1, 42, 5, 42, 
		639, 8, 42, 10, 42, 12, 42, 642, 9, 42, 3, 42, 644, 8, 42, 1, 42, 5, 42, 
		647, 8, 42, 10, 42, 12, 42, 650, 9, 42, 1, 42, 1, 42, 1, 43, 1, 43, 1, 
		43, 1, 43, 5, 43, 658, 8, 43, 10, 43, 12, 43, 661, 9, 43, 1, 43, 5, 43, 
		664, 8, 43, 10, 43, 12, 43, 667, 9, 43, 1, 44, 1, 44, 1, 45, 1, 45, 1, 
		45, 1, 45, 5, 45, 675, 8, 45, 10, 45, 12, 45, 678, 9, 45, 1, 45, 1, 45, 
		1, 45, 5, 45, 683, 8, 45, 10, 45, 12, 45, 686, 9, 45, 1, 45, 5, 45, 689, 
		8, 45, 10, 45, 12, 45, 692, 9, 45, 3, 45, 694, 8, 45, 1, 45, 5, 45, 697, 
		8, 45, 10, 45, 12, 45, 700, 9, 45, 1, 45, 1, 45, 5, 45, 704, 8, 45, 10, 
		45, 12, 45, 707, 9, 45, 1, 45, 1, 45, 3, 45, 711, 8, 45, 1, 45, 5, 45, 
		714, 8, 45, 10, 45, 12, 45, 717, 9, 45, 1, 45, 1, 45, 5, 45, 721, 8, 45, 
		10, 45, 12, 45, 724, 9, 45, 1, 45, 1, 45, 1, 45, 1, 45, 1, 45, 5, 45, 
		731, 8, 45, 10, 45, 12, 45, 734, 9, 45, 1, 45, 1, 45, 5, 45, 738, 8, 45, 
		10, 45, 12, 45, 741, 9, 45, 4, 45, 743, 8, 45, 11, 45, 12, 45, 744, 1, 
		45, 1, 45, 3, 45, 749, 8, 45, 1, 46, 1, 46, 1, 46, 1, 46, 1, 46, 3, 46, 
		756, 8, 46, 1, 47, 1, 47, 1, 47, 1, 47, 1, 47, 1, 47, 1, 47, 3, 47, 765, 
		8, 47, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 5, 48, 773, 8, 48, 10, 
		48, 12, 48, 776, 9, 48, 1, 48, 1, 48, 1, 48, 5, 48, 781, 8, 48, 10, 48, 
		12, 48, 784, 9, 48, 1, 48, 5, 48, 787, 8, 48, 10, 48, 12, 48, 790, 9, 
		48, 3, 48, 792, 8, 48, 1, 48, 5, 48, 795, 8, 48, 10, 48, 12, 48, 798, 
		9, 48, 1, 48, 1, 48, 1, 48, 1, 49, 1, 49, 1, 49, 1, 49, 1, 50, 1, 50, 
		1, 50, 5, 50, 810, 8, 50, 10, 50, 12, 50, 813, 9, 50, 1, 50, 1, 50, 1, 
		50, 5, 50, 818, 8, 50, 10, 50, 12, 50, 821, 9, 50, 1, 50, 5, 50, 824, 
		8, 50, 10, 50, 12, 50, 827, 9, 50, 3, 50, 829, 8, 50, 1, 50, 5, 50, 832, 
		8, 50, 10, 50, 12, 50, 835, 9, 50, 1, 50, 3, 50, 838, 8, 50, 1, 51, 1, 
		51, 1, 51, 1, 51, 5, 51, 844, 8, 51, 10, 51, 12, 51, 847, 9, 51, 1, 51, 
		1, 51, 5, 51, 851, 8, 51, 10, 51, 12, 51, 854, 9, 51, 5, 51, 856, 8, 51, 
		10, 51, 12, 51, 859, 9, 51, 1, 51, 1, 51, 5, 51, 863, 8, 51, 10, 51, 12, 
		51, 866, 9, 51, 1, 51, 1, 51, 1, 51, 5, 51, 871, 8, 51, 10, 51, 12, 51, 
		874, 9, 51, 1, 51, 1, 51, 5, 51, 878, 8, 51, 10, 51, 12, 51, 881, 9, 51, 
		5, 51, 883, 8, 51, 10, 51, 12, 51, 886, 9, 51, 1, 51, 3, 51, 889, 8, 51, 
		1, 52, 1, 52, 1, 52, 1, 52, 5, 52, 895, 8, 52, 10, 52, 12, 52, 898, 9, 
		52, 1, 52, 1, 52, 5, 52, 902, 8, 52, 10, 52, 12, 52, 905, 9, 52, 1, 52, 
		1, 52, 5, 52, 909, 8, 52, 10, 52, 12, 52, 912, 9, 52, 1, 52, 1, 52, 1, 
		52, 1, 52, 1, 53, 1, 53, 5, 53, 920, 8, 53, 10, 53, 12, 53, 923, 9, 53, 
		1, 53, 1, 53, 1, 53, 5, 53, 928, 8, 53, 10, 53, 12, 53, 931, 9, 53, 1, 
		53, 5, 53, 934, 8, 53, 10, 53, 12, 53, 937, 9, 53, 3, 53, 939, 8, 53, 
		1, 53, 5, 53, 942, 8, 53, 10, 53, 12, 53, 945, 9, 53, 1, 53, 1, 53, 1, 
		54, 1, 54, 1, 54, 1, 54, 5, 54, 953, 8, 54, 10, 54, 12, 54, 956, 9, 54, 
		1, 54, 5, 54, 959, 8, 54, 10, 54, 12, 54, 962, 9, 54, 1, 55, 1, 55, 1, 
		56, 1, 56, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 
		57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 
		57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 
		57, 3, 57, 997, 8, 57, 1, 58, 1, 58, 5, 58, 1001, 8, 58, 10, 58, 12, 58, 
		1004, 9, 58, 1, 58, 1, 58, 1, 58, 5, 58, 1009, 8, 58, 10, 58, 12, 58, 
		1012, 9, 58, 1, 58, 5, 58, 1015, 8, 58, 10, 58, 12, 58, 1018, 9, 58, 1, 
		58, 5, 58, 1021, 8, 58, 10, 58, 12, 58, 1024, 9, 58, 1, 58, 1, 58, 1, 
		59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 
		59, 3, 59, 1039, 8, 59, 1, 60, 1, 60, 1, 60, 5, 60, 1044, 8, 60, 10, 60, 
		12, 60, 1047, 9, 60, 1, 61, 1, 61, 1, 61, 1, 61, 1, 62, 1, 62, 1, 63, 
		1, 63, 1, 63, 5, 63, 1058, 8, 63, 10, 63, 12, 63, 1061, 9, 63, 1, 64, 
		1, 64, 1, 64, 5, 64, 1066, 8, 64, 10, 64, 12, 64, 1069, 9, 64, 1, 65, 
		1, 65, 1, 65, 1, 65, 3, 65, 1075, 8, 65, 1, 66, 1, 66, 1, 66, 5, 66, 1080, 
		8, 66, 10, 66, 12, 66, 1083, 9, 66, 1, 67, 1, 67, 1, 67, 5, 67, 1088, 
		8, 67, 10, 67, 12, 67, 1091, 9, 67, 1, 68, 1, 68, 1, 68, 3, 68, 1096, 
		8, 68, 1, 69, 1, 69, 1, 69, 1, 69, 1, 69, 5, 69, 1103, 8, 69, 10, 69, 
		12, 69, 1106, 9, 69, 3, 69, 1108, 8, 69, 1, 69, 1, 69, 1, 69, 1, 69, 1, 
		69, 1, 69, 1, 69, 1, 69, 3, 69, 1118, 8, 69, 1, 70, 1, 70, 1, 71, 1, 71, 
		1, 71, 3, 71, 1125, 8, 71, 1, 72, 1, 72, 5, 72, 1129, 8, 72, 10, 72, 12, 
		72, 1132, 9, 72, 1, 72, 1, 72, 5, 72, 1136, 8, 72, 10, 72, 12, 72, 1139, 
		9, 72, 1, 72, 1, 72, 5, 72, 1143, 8, 72, 10, 72, 12, 72, 1146, 9, 72, 
		1, 72, 1, 72, 5, 72, 1150, 8, 72, 10, 72, 12, 72, 1153, 9, 72, 1, 72, 
		1, 72, 5, 72, 1157, 8, 72, 10, 72, 12, 72, 1160, 9, 72, 1, 72, 1, 72, 
		5, 72, 1164, 8, 72, 10, 72, 12, 72, 1167, 9, 72, 1, 72, 1, 72, 5, 72, 
		1171, 8, 72, 10, 72, 12, 72, 1174, 9, 72, 1, 72, 5, 72, 1177, 8, 72, 10, 
		72, 12, 72, 1180, 9, 72, 3, 72, 1182, 8, 72, 1, 72, 5, 72, 1185, 8, 72, 
		10, 72, 12, 72, 1188, 9, 72, 1, 72, 1, 72, 1, 73, 1, 73, 5, 73, 1194, 
		8, 73, 10, 73, 12, 73, 1197, 9, 73, 1, 73, 1, 73, 5, 73, 1201, 8, 73, 
		10, 73, 12, 73, 1204, 9, 73, 1, 73, 1, 73, 5, 73, 1208, 8, 73, 10, 73, 
		12, 73, 1211, 9, 73, 1, 73, 5, 73, 1214, 8, 73, 10, 73, 12, 73, 1217, 
		9, 73, 3, 73, 1219, 8, 73, 1, 73, 5, 73, 1222, 8, 73, 10, 73, 12, 73, 
		1225, 9, 73, 1, 73, 1, 73, 1, 74, 1, 74, 1, 74, 1, 74, 1, 74, 3, 74, 1234, 
		8, 74, 1, 75, 3, 75, 1237, 8, 75, 1, 75, 1, 75, 1, 76, 1, 76, 1, 77, 4, 
		77, 1244, 8, 77, 11, 77, 12, 77, 1245, 1, 77, 3, 77, 1249, 8, 77, 1, 77, 
		0, 0, 78, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 
		34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64, 66, 68, 
		70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 90, 92, 94, 96, 98, 100, 102, 
		104, 106, 108, 110, 112, 114, 116, 118, 120, 122, 124, 126, 128, 130, 
		132, 134, 136, 138, 140, 142, 144, 146, 148, 150, 152, 154, 0, 8, 2, 0, 
		24, 24, 47, 48, 2, 0, 24, 24, 48, 48, 5, 0, 28, 28, 30, 30, 37, 37, 40, 
		40, 57, 57, 1, 0, 70, 71, 1, 0, 72, 74, 2, 0, 70, 71, 75, 75, 1, 0, 76, 
		81, 14, 0, 2, 2, 5, 7, 16, 20, 23, 23, 25, 27, 33, 36, 38, 39, 42, 42, 
		44, 44, 49, 49, 52, 52, 54, 56, 59, 65, 85, 85, 1362, 0, 159, 1, 0, 0, 
		0, 2, 194, 1, 0, 0, 0, 4, 196, 1, 0, 0, 0, 6, 219, 1, 0, 0, 0, 8, 225, 
		1, 0, 0, 0, 10, 285, 1, 0, 0, 0, 12, 287, 1, 0, 0, 0, 14, 303, 1, 0, 0, 
		0, 16, 320, 1, 0, 0, 0, 18, 324, 1, 0, 0, 0, 20, 349, 1, 0, 0, 0, 22, 
		351, 1, 0, 0, 0, 24, 355, 1, 0, 0, 0, 26, 359, 1, 0, 0, 0, 28, 363, 1, 
		0, 0, 0, 30, 367, 1, 0, 0, 0, 32, 371, 1, 0, 0, 0, 34, 378, 1, 0, 0, 0, 
		36, 382, 1, 0, 0, 0, 38, 404, 1, 0, 0, 0, 40, 406, 1, 0, 0, 0, 42, 410, 
		1, 0, 0, 0, 44, 414, 1, 0, 0, 0, 46, 418, 1, 0, 0, 0, 48, 422, 1, 0, 0, 
		0, 50, 444, 1, 0, 0, 0, 52, 446, 1, 0, 0, 0, 54, 450, 1, 0, 0, 0, 56, 
		454, 1, 0, 0, 0, 58, 458, 1, 0, 0, 0, 60, 469, 1, 0, 0, 0, 62, 489, 1, 
		0, 0, 0, 64, 491, 1, 0, 0, 0, 66, 520, 1, 0, 0, 0, 68, 522, 1, 0, 0, 0, 
		70, 526, 1, 0, 0, 0, 72, 530, 1, 0, 0, 0, 74, 534, 1, 0, 0, 0, 76, 538, 
		1, 0, 0, 0, 78, 542, 1, 0, 0, 0, 80, 546, 1, 0, 0, 0, 82, 583, 1, 0, 0, 
		0, 84, 622, 1, 0, 0, 0, 86, 653, 1, 0, 0, 0, 88, 668, 1, 0, 0, 0, 90, 
		670, 1, 0, 0, 0, 92, 750, 1, 0, 0, 0, 94, 764, 1, 0, 0, 0, 96, 766, 1, 
		0, 0, 0, 98, 802, 1, 0, 0, 0, 100, 837, 1, 0, 0, 0, 102, 839, 1, 0, 0, 
		0, 104, 890, 1, 0, 0, 0, 106, 917, 1, 0, 0, 0, 108, 948, 1, 0, 0, 0, 110, 
		963, 1, 0, 0, 0, 112, 965, 1, 0, 0, 0, 114, 996, 1, 0, 0, 0, 116, 998, 
		1, 0, 0, 0, 118, 1038, 1, 0, 0, 0, 120, 1040, 1, 0, 0, 0, 122, 1048, 1, 
		0, 0, 0, 124, 1052, 1, 0, 0, 0, 126, 1054, 1, 0, 0, 0, 128, 1062, 1, 0, 
		0, 0, 130, 1070, 1, 0, 0, 0, 132, 1076, 1, 0, 0, 0, 134, 1084, 1, 0, 0, 
		0, 136, 1095, 1, 0, 0, 0, 138, 1117, 1, 0, 0, 0, 140, 1119, 1, 0, 0, 0, 
		142, 1124, 1, 0, 0, 0, 144, 1126, 1, 0, 0, 0, 146, 1191, 1, 0, 0, 0, 148, 
		1233, 1, 0, 0, 0, 150, 1236, 1, 0, 0, 0, 152, 1240, 1, 0, 0, 0, 154, 1248, 
		1, 0, 0, 0, 156, 158, 5, 90, 0, 0, 157, 156, 1, 0, 0, 0, 158, 161, 1, 
		0, 0, 0, 159, 157, 1, 0, 0, 0, 159, 160, 1, 0, 0, 0, 160, 162, 1, 0, 0, 
		0, 161, 159, 1, 0, 0, 0, 162, 163, 5, 1, 0, 0, 163, 164, 5, 87, 0, 0, 
		164, 174, 3, 154, 77, 0, 165, 169, 3, 2, 1, 0, 166, 168, 5, 90, 0, 0, 
		167, 166, 1, 0, 0, 0, 168, 171, 1, 0, 0, 0, 169, 167, 1, 0, 0, 0, 169, 
		170, 1, 0, 0, 0, 170, 173, 1, 0, 0, 0, 171, 169, 1, 0, 0, 0, 172, 165, 
		1, 0, 0, 0, 173, 176, 1, 0, 0, 0, 174, 172, 1, 0, 0, 0, 174, 175, 1, 0, 
		0, 0, 175, 177, 1, 0, 0, 0, 176, 174, 1, 0, 0, 0, 177, 178, 5, 0, 0, 1, 
		178, 1, 1, 0, 0, 0, 179, 195, 3, 8, 4, 0, 180, 195, 3, 4, 2, 0, 181, 195, 
		3, 6, 3, 0, 182, 195, 3, 12, 6, 0, 183, 195, 3, 14, 7, 0, 184, 195, 3, 
		18, 9, 0, 185, 195, 3, 36, 18, 0, 186, 195, 3, 48, 24, 0, 187, 195, 3, 
		60, 30, 0, 188, 195, 3, 80, 40, 0, 189, 195, 3, 104, 52, 0, 190, 195, 
		3, 90, 45, 0, 191, 195, 3, 96, 48, 0, 192, 195, 3, 102, 51, 0, 193, 195, 
		3, 114, 57, 0, 194, 179, 1, 0, 0, 0, 194, 180, 1, 0, 0, 0, 194, 181, 1, 
		0, 0, 0, 194, 182, 1, 0, 0, 0, 194, 183, 1, 0, 0, 0, 194, 184, 1, 0, 0, 
		0, 194, 185, 1, 0, 0, 0, 194, 186, 1, 0, 0, 0, 194, 187, 1, 0, 0, 0, 194, 
		188, 1, 0, 0, 0, 194, 189, 1, 0, 0, 0, 194, 190, 1, 0, 0, 0, 194, 191, 
		1, 0, 0, 0, 194, 192, 1, 0, 0, 0, 194, 193, 1, 0, 0, 0, 195, 3, 1, 0, 
		0, 0, 196, 197, 5, 2, 0, 0, 197, 198, 3, 120, 60, 0, 198, 202, 5, 3, 0, 
		0, 199, 201, 5, 90, 0, 0, 200, 199, 1, 0, 0, 0, 201, 204, 1, 0, 0, 0, 
		202, 200, 1, 0, 0, 0, 202, 203, 1, 0, 0, 0, 203, 214, 1, 0, 0, 0, 204, 
		202, 1, 0, 0, 0, 205, 209, 3, 2, 1, 0, 206, 208, 5, 90, 0, 0, 207, 206, 
		1, 0, 0, 0, 208, 211, 1, 0, 0, 0, 209, 207, 1, 0, 0, 0, 209, 210, 1, 0, 
		0, 0, 210, 213, 1, 0, 0, 0, 211, 209, 1, 0, 0, 0, 212, 205, 1, 0, 0, 0, 
		213, 216, 1, 0, 0, 0, 214, 212, 1, 0, 0, 0, 214, 215, 1, 0, 0, 0, 215, 
		217, 1, 0, 0, 0, 216, 214, 1, 0, 0, 0, 217, 218, 5, 4, 0, 0, 218, 5, 1, 
		0, 0, 0, 219, 220, 5, 5, 0, 0, 220, 221, 5, 87, 0, 0, 221, 222, 5, 6, 
		0, 0, 222, 223, 3, 152, 76, 0, 223, 224, 3, 154, 77, 0, 224, 7, 1, 0, 
		0, 0, 225, 226, 5, 7, 0, 0, 226, 231, 3, 10, 5, 0, 227, 228, 5, 8, 0, 
		0, 228, 230, 3, 10, 5, 0, 229, 227, 1, 0, 0, 0, 230, 233, 1, 0, 0, 0, 
		231, 229, 1, 0, 0, 0, 231, 232, 1, 0, 0, 0, 232, 234, 1, 0, 0, 0, 233, 
		231, 1, 0, 0, 0, 234, 238, 5, 3, 0, 0, 235, 237, 5, 90, 0, 0, 236, 235, 
		1, 0, 0, 0, 237, 240, 1, 0, 0, 0, 238, 236, 1, 0, 0, 0, 238, 239, 1, 0, 
		0, 0, 239, 250, 1, 0, 0, 0, 240, 238, 1, 0, 0, 0, 241, 245, 3, 2, 1, 0, 
		242, 244, 5, 90, 0, 0, 243, 242, 1, 0, 0, 0, 244, 247, 1, 0, 0, 0, 245, 
		243, 1, 0, 0, 0, 245, 246, 1, 0, 0, 0, 246, 249, 1, 0, 0, 0, 247, 245, 
		1, 0, 0, 0, 248, 241, 1, 0, 0, 0, 249, 252, 1, 0, 0, 0, 250, 248, 1, 0, 
		0, 0, 250, 251, 1, 0, 0, 0, 251, 253, 1, 0, 0, 0, 252, 250, 1, 0, 0, 0, 
		253, 254, 5, 4, 0, 0, 254, 9, 1, 0, 0, 0, 255, 256, 5, 9, 0, 0, 256, 286, 
		3, 120, 60, 0, 257, 258, 5, 10, 0, 0, 258, 286, 3, 120, 60, 0, 259, 260, 
		5, 11, 0, 0, 260, 286, 3, 150, 75, 0, 261, 262, 5, 12, 0, 0, 262, 286, 
		3, 152, 76, 0, 263, 264, 5, 13, 0, 0, 264, 286, 3, 142, 71, 0, 265, 266, 
		5, 14, 0, 0, 266, 286, 5, 87, 0, 0, 267, 268, 5, 15, 0, 0, 268, 286, 3, 
		120, 60, 0, 269, 270, 5, 16, 0, 0, 270, 286, 5, 87, 0, 0, 271, 272, 5, 
		17, 0, 0, 272, 286, 5, 87, 0, 0, 273, 274, 5, 18, 0, 0, 274, 286, 5, 87, 
		0, 0, 275, 276, 5, 19, 0, 0, 276, 286, 5, 87, 0, 0, 277, 278, 5, 20, 0, 
		0, 278, 286, 5, 87, 0, 0, 279, 280, 5, 21, 0, 0, 280, 286, 5, 87, 0, 0, 
		281, 282, 5, 22, 0, 0, 282, 286, 5, 87, 0, 0, 283, 284, 5, 23, 0, 0, 284, 
		286, 3, 142, 71, 0, 285, 255, 1, 0, 0, 0, 285, 257, 1, 0, 0, 0, 285, 259, 
		1, 0, 0, 0, 285, 261, 1, 0, 0, 0, 285, 263, 1, 0, 0, 0, 285, 265, 1, 0, 
		0, 0, 285, 267, 1, 0, 0, 0, 285, 269, 1, 0, 0, 0, 285, 271, 1, 0, 0, 0, 
		285, 273, 1, 0, 0, 0, 285, 275, 1, 0, 0, 0, 285, 277, 1, 0, 0, 0, 285, 
		279, 1, 0, 0, 0, 285, 281, 1, 0, 0, 0, 285, 283, 1, 0, 0, 0, 286, 11, 
		1, 0, 0, 0, 287, 288, 5, 9, 0, 0, 288, 289, 3, 120, 60, 0, 289, 293, 5, 
		3, 0, 0, 290, 292, 5, 90, 0, 0, 291, 290, 1, 0, 0, 0, 292, 295, 1, 0, 
		0, 0, 293, 291, 1, 0, 0, 0, 293, 294, 1, 0, 0, 0, 294, 297, 1, 0, 0, 0, 
		295, 293, 1, 0, 0, 0, 296, 298, 3, 122, 61, 0, 297, 296, 1, 0, 0, 0, 298, 
		299, 1, 0, 0, 0, 299, 297, 1, 0, 0, 0, 299, 300, 1, 0, 0, 0, 300, 301, 
		1, 0, 0, 0, 301, 302, 5, 4, 0, 0, 302, 13, 1, 0, 0, 0, 303, 304, 5, 10, 
		0, 0, 304, 305, 3, 120, 60, 0, 305, 309, 5, 3, 0, 0, 306, 308, 5, 90, 
		0, 0, 307, 306, 1, 0, 0, 0, 308, 311, 1, 0, 0, 0, 309, 307, 1, 0, 0, 0, 
		309, 310, 1, 0, 0, 0, 310, 315, 1, 0, 0, 0, 311, 309, 1, 0, 0, 0, 312, 
		314, 3, 16, 8, 0, 313, 312, 1, 0, 0, 0, 314, 317, 1, 0, 0, 0, 315, 313, 
		1, 0, 0, 0, 315, 316, 1, 0, 0, 0, 316, 318, 1, 0, 0, 0, 317, 315, 1, 0, 
		0, 0, 318, 319, 5, 4, 0, 0, 319, 15, 1, 0, 0, 0, 320, 321, 5, 14, 0, 0, 
		321, 322, 5, 87, 0, 0, 322, 323, 3, 154, 77, 0, 323, 17, 1, 0, 0, 0, 324, 
		325, 5, 24, 0, 0, 325, 326, 3, 120, 60, 0, 326, 330, 5, 3, 0, 0, 327, 
		329, 5, 90, 0, 0, 328, 327, 1, 0, 0, 0, 329, 332, 1, 0, 0, 0, 330, 328, 
		1, 0, 0, 0, 330, 331, 1, 0, 0, 0, 331, 336, 1, 0, 0, 0, 332, 330, 1, 0, 
		0, 0, 333, 335, 3, 20, 10, 0, 334, 333, 1, 0, 0, 0, 335, 338, 1, 0, 0, 
		0, 336, 334, 1, 0, 0, 0, 336, 337, 1, 0, 0, 0, 337, 339, 1, 0, 0, 0, 338, 
		336, 1, 0, 0, 0, 339, 340, 5, 4, 0, 0, 340, 19, 1, 0, 0, 0, 341, 350, 
		3, 22, 11, 0, 342, 350, 3, 34, 17, 0, 343, 350, 3, 24, 12, 0, 344, 350, 
		3, 26, 13, 0, 345, 350, 3, 28, 14, 0, 346, 350, 3, 30, 15, 0, 347, 350, 
		3, 32, 16, 0, 348, 350, 3, 122, 61, 0, 349, 341, 1, 0, 0, 0, 349, 342, 
		1, 0, 0, 0, 349, 343, 1, 0, 0, 0, 349, 344, 1, 0, 0, 0, 349, 345, 1, 0, 
		0, 0, 349, 346, 1, 0, 0, 0, 349, 347, 1, 0, 0, 0, 349, 348, 1, 0, 0, 0, 
		350, 21, 1, 0, 0, 0, 351, 352, 5, 10, 0, 0, 352, 353, 3, 120, 60, 0, 353, 
		354, 3, 154, 77, 0, 354, 23, 1, 0, 0, 0, 355, 356, 5, 12, 0, 0, 356, 357, 
		3, 152, 76, 0, 357, 358, 3, 154, 77, 0, 358, 25, 1, 0, 0, 0, 359, 360, 
		5, 9, 0, 0, 360, 361, 3, 120, 60, 0, 361, 362, 3, 154, 77, 0, 362, 27, 
		1, 0, 0, 0, 363, 364, 5, 11, 0, 0, 364, 365, 3, 150, 75, 0, 365, 366, 
		3, 154, 77, 0, 366, 29, 1, 0, 0, 0, 367, 368, 5, 13, 0, 0, 368, 369, 3, 
		142, 71, 0, 369, 370, 3, 154, 77, 0, 370, 31, 1, 0, 0, 0, 371, 372, 5, 
		25, 0, 0, 372, 373, 5, 87, 0, 0, 373, 374, 5, 26, 0, 0, 374, 375, 3, 120, 
		60, 0, 375, 376, 5, 87, 0, 0, 376, 377, 3, 154, 77, 0, 377, 33, 1, 0, 
		0, 0, 378, 379, 5, 27, 0, 0, 379, 380, 3, 124, 62, 0, 380, 381, 3, 154, 
		77, 0, 381, 35, 1, 0, 0, 0, 382, 383, 5, 28, 0, 0, 383, 384, 3, 120, 60, 
		0, 384, 388, 5, 3, 0, 0, 385, 387, 5, 90, 0, 0, 386, 385, 1, 0, 0, 0, 
		387, 390, 1, 0, 0, 0, 388, 386, 1, 0, 0, 0, 388, 389, 1, 0, 0, 0, 389, 
		394, 1, 0, 0, 0, 390, 388, 1, 0, 0, 0, 391, 393, 3, 38, 19, 0, 392, 391, 
		1, 0, 0, 0, 393, 396, 1, 0, 0, 0, 394, 392, 1, 0, 0, 0, 394, 395, 1, 0, 
		0, 0, 395, 397, 1, 0, 0, 0, 396, 394, 1, 0, 0, 0, 397, 398, 5, 4, 0, 0, 
		398, 37, 1, 0, 0, 0, 399, 405, 3, 40, 20, 0, 400, 405, 3, 26, 13, 0, 401, 
		405, 3, 42, 21, 0, 402, 405, 3, 44, 22, 0, 403, 405, 3, 46, 23, 0, 404, 
		399, 1, 0, 0, 0, 404, 400, 1, 0, 0, 0, 404, 401, 1, 0, 0, 0, 404, 402, 
		1, 0, 0, 0, 404, 403, 1, 0, 0, 0, 405, 39, 1, 0, 0, 0, 406, 407, 5, 29, 
		0, 0, 407, 408, 5, 87, 0, 0, 408, 409, 3, 154, 77, 0, 409, 41, 1, 0, 0, 
		0, 410, 411, 5, 15, 0, 0, 411, 412, 3, 120, 60, 0, 412, 413, 3, 154, 77, 
		0, 413, 43, 1, 0, 0, 0, 414, 415, 5, 21, 0, 0, 415, 416, 5, 87, 0, 0, 
		416, 417, 3, 154, 77, 0, 417, 45, 1, 0, 0, 0, 418, 419, 5, 22, 0, 0, 419, 
		420, 5, 87, 0, 0, 420, 421, 3, 154, 77, 0, 421, 47, 1, 0, 0, 0, 422, 423, 
		5, 30, 0, 0, 423, 424, 3, 120, 60, 0, 424, 428, 5, 3, 0, 0, 425, 427, 
		5, 90, 0, 0, 426, 425, 1, 0, 0, 0, 427, 430, 1, 0, 0, 0, 428, 426, 1, 
		0, 0, 0, 428, 429, 1, 0, 0, 0, 429, 434, 1, 0, 0, 0, 430, 428, 1, 0, 0, 
		0, 431, 433, 3, 50, 25, 0, 432, 431, 1, 0, 0, 0, 433, 436, 1, 0, 0, 0, 
		434, 432, 1, 0, 0, 0, 434, 435, 1, 0, 0, 0, 435, 437, 1, 0, 0, 0, 436, 
		434, 1, 0, 0, 0, 437, 438, 5, 4, 0, 0, 438, 49, 1, 0, 0, 0, 439, 445, 
		3, 52, 26, 0, 440, 445, 3, 54, 27, 0, 441, 445, 3, 56, 28, 0, 442, 445, 
		3, 58, 29, 0, 443, 445, 3, 122, 61, 0, 444, 439, 1, 0, 0, 0, 444, 440, 
		1, 0, 0, 0, 444, 441, 1, 0, 0, 0, 444, 442, 1, 0, 0, 0, 444, 443, 1, 0, 
		0, 0, 445, 51, 1, 0, 0, 0, 446, 447, 5, 16, 0, 0, 447, 448, 5, 87, 0, 
		0, 448, 449, 3, 154, 77, 0, 449, 53, 1, 0, 0, 0, 450, 451, 5, 31, 0, 0, 
		451, 452, 5, 87, 0, 0, 452, 453, 3, 154, 77, 0, 453, 55, 1, 0, 0, 0, 454, 
		455, 5, 32, 0, 0, 455, 456, 3, 116, 58, 0, 456, 457, 3, 154, 77, 0, 457, 
		57, 1, 0, 0, 0, 458, 459, 5, 33, 0, 0, 459, 460, 5, 26, 0, 0, 460, 461, 
		3, 120, 60, 0, 461, 462, 5, 34, 0, 0, 462, 463, 3, 120, 60, 0, 463, 464, 
		5, 35, 0, 0, 464, 465, 3, 120, 60, 0, 465, 466, 5, 36, 0, 0, 466, 467, 
		5, 87, 0, 0, 467, 468, 3, 154, 77, 0, 468, 59, 1, 0, 0, 0, 469, 470, 5, 
		37, 0, 0, 470, 471, 3, 120, 60, 0, 471, 475, 5, 3, 0, 0, 472, 474, 5, 
		90, 0, 0, 473, 472, 1, 0, 0, 0, 474, 477, 1, 0, 0, 0, 475, 473, 1, 0, 
		0, 0, 475, 476, 1, 0, 0, 0, 476, 481, 1, 0, 0, 0, 477, 475, 1, 0, 0, 0, 
		478, 480, 3, 62, 31, 0, 479, 478, 1, 0, 0, 0, 480, 483, 1, 0, 0, 0, 481, 
		479, 1, 0, 0, 0, 481, 482, 1, 0, 0, 0, 482, 484, 1, 0, 0, 0, 483, 481, 
		1, 0, 0, 0, 484, 485, 5, 4, 0, 0, 485, 61, 1, 0, 0, 0, 486, 490, 3, 40, 
		20, 0, 487, 490, 3, 26, 13, 0, 488, 490, 3, 64, 32, 0, 489, 486, 1, 0, 
		0, 0, 489, 487, 1, 0, 0, 0, 489, 488, 1, 0, 0, 0, 490, 63, 1, 0, 0, 0, 
		491, 492, 5, 38, 0, 0, 492, 496, 5, 3, 0, 0, 493, 495, 5, 90, 0, 0, 494, 
		493, 1, 0, 0, 0, 495, 498, 1, 0, 0, 0, 496, 494, 1, 0, 0, 0, 496, 497, 
		1, 0, 0, 0, 497, 502, 1, 0, 0, 0, 498, 496, 1, 0, 0, 0, 499, 501, 3, 66, 
		33, 0, 500, 499, 1, 0, 0, 0, 501, 504, 1, 0, 0, 0, 502, 500, 1, 0, 0, 
		0, 502, 503, 1, 0, 0, 0, 503, 505, 1, 0, 0, 0, 504, 502, 1, 0, 0, 0, 505, 
		509, 5, 4, 0, 0, 506, 508, 5, 90, 0, 0, 507, 506, 1, 0, 0, 0, 508, 511, 
		1, 0, 0, 0, 509, 507, 1, 0, 0, 0, 509, 510, 1, 0, 0, 0, 510, 65, 1, 0, 
		0, 0, 511, 509, 1, 0, 0, 0, 512, 521, 3, 68, 34, 0, 513, 521, 3, 70, 35, 
		0, 514, 521, 3, 72, 36, 0, 515, 521, 3, 74, 37, 0, 516, 521, 3, 44, 22, 
		0, 517, 521, 3, 46, 23, 0, 518, 521, 3, 76, 38, 0, 519, 521, 3, 78, 39, 
		0, 520, 512, 1, 0, 0, 0, 520, 513, 1, 0, 0, 0, 520, 514, 1, 0, 0, 0, 520, 
		515, 1, 0, 0, 0, 520, 516, 1, 0, 0, 0, 520, 517, 1, 0, 0, 0, 520, 518, 
		1, 0, 0, 0, 520, 519, 1, 0, 0, 0, 521, 67, 1, 0, 0, 0, 522, 523, 5, 17, 
		0, 0, 523, 524, 5, 87, 0, 0, 524, 525, 3, 154, 77, 0, 525, 69, 1, 0, 0, 
		0, 526, 527, 5, 18, 0, 0, 527, 528, 5, 87, 0, 0, 528, 529, 3, 154, 77, 
		0, 529, 71, 1, 0, 0, 0, 530, 531, 5, 19, 0, 0, 531, 532, 5, 87, 0, 0, 
		532, 533, 3, 154, 77, 0, 533, 73, 1, 0, 0, 0, 534, 535, 5, 20, 0, 0, 535, 
		536, 5, 87, 0, 0, 536, 537, 3, 154, 77, 0, 537, 75, 1, 0, 0, 0, 538, 539, 
		5, 23, 0, 0, 539, 540, 3, 142, 71, 0, 540, 541, 3, 154, 77, 0, 541, 77, 
		1, 0, 0, 0, 542, 543, 5, 39, 0, 0, 543, 544, 3, 124, 62, 0, 544, 545, 
		3, 154, 77, 0, 545, 79, 1, 0, 0, 0, 546, 547, 5, 40, 0, 0, 547, 581, 3, 
		120, 60, 0, 548, 552, 5, 41, 0, 0, 549, 551, 5, 90, 0, 0, 550, 549, 1, 
		0, 0, 0, 551, 554, 1, 0, 0, 0, 552, 550, 1, 0, 0, 0, 552, 553, 1, 0, 0, 
		0, 553, 555, 1, 0, 0, 0, 554, 552, 1, 0, 0, 0, 555, 556, 3, 82, 41, 0, 
		556, 557, 3, 154, 77, 0, 557, 582, 1, 0, 0, 0, 558, 562, 5, 3, 0, 0, 559, 
		561, 5, 90, 0, 0, 560, 559, 1, 0, 0, 0, 561, 564, 1, 0, 0, 0, 562, 560, 
		1, 0, 0, 0, 562, 563, 1, 0, 0, 0, 563, 574, 1, 0, 0, 0, 564, 562, 1, 0, 
		0, 0, 565, 569, 3, 2, 1, 0, 566, 568, 5, 90, 0, 0, 567, 566, 1, 0, 0, 
		0, 568, 571, 1, 0, 0, 0, 569, 567, 1, 0, 0, 0, 569, 570, 1, 0, 0, 0, 570, 
		573, 1, 0, 0, 0, 571, 569, 1, 0, 0, 0, 572, 565, 1, 0, 0, 0, 573, 576, 
		1, 0, 0, 0, 574, 572, 1, 0, 0, 0, 574, 575, 1, 0, 0, 0, 575, 577, 1, 0, 
		0, 0, 576, 574, 1, 0, 0, 0, 577, 578, 3, 82, 41, 0, 578, 579, 3, 154, 
		77, 0, 579, 580, 5, 4, 0, 0, 580, 582, 1, 0, 0, 0, 581, 548, 1, 0, 0, 
		0, 581, 558, 1, 0, 0, 0, 582, 81, 1, 0, 0, 0, 583, 587, 3, 84, 42, 0, 
		584, 586, 5, 90, 0, 0, 585, 584, 1, 0, 0, 0, 586, 589, 1, 0, 0, 0, 587, 
		585, 1, 0, 0, 0, 587, 588, 1, 0, 0, 0, 588, 590, 1, 0, 0, 0, 589, 587, 
		1, 0, 0, 0, 590, 594, 5, 42, 0, 0, 591, 593, 5, 90, 0, 0, 592, 591, 1, 
		0, 0, 0, 593, 596, 1, 0, 0, 0, 594, 592, 1, 0, 0, 0, 594, 595, 1, 0, 0, 
		0, 595, 597, 1, 0, 0, 0, 596, 594, 1, 0, 0, 0, 597, 601, 3, 118, 59, 0, 
		598, 600, 5, 90, 0, 0, 599, 598, 1, 0, 0, 0, 600, 603, 1, 0, 0, 0, 601, 
		599, 1, 0, 0, 0, 601, 602, 1, 0, 0, 0, 602, 604, 1, 0, 0, 0, 603, 601, 
		1, 0, 0, 0, 604, 608, 5, 43, 0, 0, 605, 607, 5, 90, 0, 0, 606, 605, 1, 
		0, 0, 0, 607, 610, 1, 0, 0, 0, 608, 606, 1, 0, 0, 0, 608, 609, 1, 0, 0, 
		0, 609, 611, 1, 0, 0, 0, 610, 608, 1, 0, 0, 0, 611, 620, 3, 118, 59, 0, 
		612, 614, 5, 90, 0, 0, 613, 612, 1, 0, 0, 0, 614, 617, 1, 0, 0, 0, 615, 
		613, 1, 0, 0, 0, 615, 616, 1, 0, 0, 0, 616, 618, 1, 0, 0, 0, 617, 615, 
		1, 0, 0, 0, 618, 619, 5, 44, 0, 0, 619, 621, 3, 118, 59, 0, 620, 615, 
		1, 0, 0, 0, 620, 621, 1, 0, 0, 0, 621, 83, 1, 0, 0, 0, 622, 626, 5, 45, 
		0, 0, 623, 625, 5, 90, 0, 0, 624, 623, 1, 0, 0, 0, 625, 628, 1, 0, 0, 
		0, 626, 624, 1, 0, 0, 0, 626, 627, 1, 0, 0, 0, 627, 643, 1, 0, 0, 0, 628, 
		626, 1, 0, 0, 0, 629, 640, 3, 86, 43, 0, 630, 634, 5, 8, 0, 0, 631, 633, 
		5, 90, 0, 0, 632, 631, 1, 0, 0, 0, 633, 636, 1, 0, 0, 0, 634, 632, 1, 
		0, 0, 0, 634, 635, 1, 0, 0, 0, 635, 637, 1, 0, 0, 0, 636, 634, 1, 0, 0, 
		0, 637, 639, 3, 86, 43, 0, 638, 630, 1, 0, 0, 0, 639, 642, 1, 0, 0, 0, 
		640, 638, 1, 0, 0, 0, 640, 641, 1, 0, 0, 0, 641, 644, 1, 0, 0, 0, 642, 
		640, 1, 0, 0, 0, 643, 629, 1, 0, 0, 0, 643, 644, 1, 0, 0, 0, 644, 648, 
		1, 0, 0, 0, 645, 647, 5, 90, 0, 0, 646, 645, 1, 0, 0, 0, 647, 650, 1, 
		0, 0, 0, 648, 646, 1, 0, 0, 0, 648, 649, 1, 0, 0, 0, 649, 651, 1, 0, 0, 
		0, 650, 648, 1, 0, 0, 0, 651, 652, 5, 46, 0, 0, 652, 85, 1, 0, 0, 0, 653, 
		654, 3, 88, 44, 0, 654, 665, 3, 118, 59, 0, 655, 659, 5, 8, 0, 0, 656, 
		658, 5, 90, 0, 0, 657, 656, 1, 0, 0, 0, 658, 661, 1, 0, 0, 0, 659, 657, 
		1, 0, 0, 0, 659, 660, 1, 0, 0, 0, 660, 662, 1, 0, 0, 0, 661, 659, 1, 0, 
		0, 0, 662, 664, 3, 118, 59, 0, 663, 655, 1, 0, 0, 0, 664, 667, 1, 0, 0, 
		0, 665, 663, 1, 0, 0, 0, 665, 666, 1, 0, 0, 0, 666, 87, 1, 0, 0, 0, 667, 
		665, 1, 0, 0, 0, 668, 669, 7, 0, 0, 0, 669, 89, 1, 0, 0, 0, 670, 671, 
		5, 49, 0, 0, 671, 672, 3, 120, 60, 0, 672, 676, 5, 50, 0, 0, 673, 675, 
		5, 90, 0, 0, 674, 673, 1, 0, 0, 0, 675, 678, 1, 0, 0, 0, 676, 674, 1, 
		0, 0, 0, 676, 677, 1, 0, 0, 0, 677, 693, 1, 0, 0, 0, 678, 676, 1, 0, 0, 
		0, 679, 690, 3, 92, 46, 0, 680, 684, 5, 8, 0, 0, 681, 683, 5, 90, 0, 0, 
		682, 681, 1, 0, 0, 0, 683, 686, 1, 0, 0, 0, 684, 682, 1, 0, 0, 0, 684, 
		685, 1, 0, 0, 0, 685, 687, 1, 0, 0, 0, 686, 684, 1, 0, 0, 0, 687, 689, 
		3, 92, 46, 0, 688, 680, 1, 0, 0, 0, 689, 692, 1, 0, 0, 0, 690, 688, 1, 
		0, 0, 0, 690, 691, 1, 0, 0, 0, 691, 694, 1, 0, 0, 0, 692, 690, 1, 0, 0, 
		0, 693, 679, 1, 0, 0, 0, 693, 694, 1, 0, 0, 0, 694, 698, 1, 0, 0, 0, 695, 
		697, 5, 90, 0, 0, 696, 695, 1, 0, 0, 0, 697, 700, 1, 0, 0, 0, 698, 696, 
		1, 0, 0, 0, 698, 699, 1, 0, 0, 0, 699, 701, 1, 0, 0, 0, 700, 698, 1, 0, 
		0, 0, 701, 710, 5, 51, 0, 0, 702, 704, 5, 90, 0, 0, 703, 702, 1, 0, 0, 
		0, 704, 707, 1, 0, 0, 0, 705, 703, 1, 0, 0, 0, 705, 706, 1, 0, 0, 0, 706, 
		708, 1, 0, 0, 0, 707, 705, 1, 0, 0, 0, 708, 709, 5, 52, 0, 0, 709, 711, 
		3, 152, 76, 0, 710, 705, 1, 0, 0, 0, 710, 711, 1, 0, 0, 0, 711, 715, 1, 
		0, 0, 0, 712, 714, 5, 90, 0, 0, 713, 712, 1, 0, 0, 0, 714, 717, 1, 0, 
		0, 0, 715, 713, 1, 0, 0, 0, 715, 716, 1, 0, 0, 0, 716, 748, 1, 0, 0, 0, 
		717, 715, 1, 0, 0, 0, 718, 722, 5, 41, 0, 0, 719, 721, 5, 90, 0, 0, 720, 
		719, 1, 0, 0, 0, 721, 724, 1, 0, 0, 0, 722, 720, 1, 0, 0, 0, 722, 723, 
		1, 0, 0, 0, 723, 725, 1, 0, 0, 0, 724, 722, 1, 0, 0, 0, 725, 726, 3, 82, 
		41, 0, 726, 727, 3, 154, 77, 0, 727, 749, 1, 0, 0, 0, 728, 732, 5, 3, 
		0, 0, 729, 731, 5, 90, 0, 0, 730, 729, 1, 0, 0, 0, 731, 734, 1, 0, 0, 
		0, 732, 730, 1, 0, 0, 0, 732, 733, 1, 0, 0, 0, 733, 742, 1, 0, 0, 0, 734, 
		732, 1, 0, 0, 0, 735, 739, 3, 2, 1, 0, 736, 738, 5, 90, 0, 0, 737, 736, 
		1, 0, 0, 0, 738, 741, 1, 0, 0, 0, 739, 737, 1, 0, 0, 0, 739, 740, 1, 0, 
		0, 0, 740, 743, 1, 0, 0, 0, 741, 739, 1, 0, 0, 0, 742, 735, 1, 0, 0, 0, 
		743, 744, 1, 0, 0, 0, 744, 742, 1, 0, 0, 0, 744, 745, 1, 0, 0, 0, 745, 
		746, 1, 0, 0, 0, 746, 747, 5, 4, 0, 0, 747, 749, 1, 0, 0, 0, 748, 718, 
		1, 0, 0, 0, 748, 728, 1, 0, 0, 0, 749, 91, 1, 0, 0, 0, 750, 751, 3, 152, 
		76, 0, 751, 752, 5, 53, 0, 0, 752, 755, 3, 94, 47, 0, 753, 754, 5, 45, 
		0, 0, 754, 756, 5, 46, 0, 0, 755, 753, 1, 0, 0, 0, 755, 756, 1, 0, 0, 
		0, 756, 93, 1, 0, 0, 0, 757, 765, 5, 37, 0, 0, 758, 765, 5, 30, 0, 0, 
		759, 765, 5, 24, 0, 0, 760, 765, 5, 28, 0, 0, 761, 765, 5, 9, 0, 0, 762, 
		765, 5, 10, 0, 0, 763, 765, 3, 152, 76, 0, 764, 757, 1, 0, 0, 0, 764, 
		758, 1, 0, 0, 0, 764, 759, 1, 0, 0, 0, 764, 760, 1, 0, 0, 0, 764, 761, 
		1, 0, 0, 0, 764, 762, 1, 0, 0, 0, 764, 763, 1, 0, 0, 0, 765, 95, 1, 0, 
		0, 0, 766, 767, 5, 54, 0, 0, 767, 768, 3, 120, 60, 0, 768, 769, 5, 41, 
		0, 0, 769, 770, 3, 120, 60, 0, 770, 774, 5, 50, 0, 0, 771, 773, 5, 90, 
		0, 0, 772, 771, 1, 0, 0, 0, 773, 776, 1, 0, 0, 0, 774, 772, 1, 0, 0, 0, 
		774, 775, 1, 0, 0, 0, 775, 791, 1, 0, 0, 0, 776, 774, 1, 0, 0, 0, 777, 
		788, 3, 98, 49, 0, 778, 782, 5, 8, 0, 0, 779, 781, 5, 90, 0, 0, 780, 779, 
		1, 0, 0, 0, 781, 784, 1, 0, 0, 0, 782, 780, 1, 0, 0, 0, 782, 783, 1, 0, 
		0, 0, 783, 785, 1, 0, 0, 0, 784, 782, 1, 0, 0, 0, 785, 787, 3, 98, 49, 
		0, 786, 778, 1, 0, 0, 0, 787, 790, 1, 0, 0, 0, 788, 786, 1, 0, 0, 0, 788, 
		789, 1, 0, 0, 0, 789, 792, 1, 0, 0, 0, 790, 788, 1, 0, 0, 0, 791, 777, 
		1, 0, 0, 0, 791, 792, 1, 0, 0, 0, 792, 796, 1, 0, 0, 0, 793, 795, 5, 90, 
		0, 0, 794, 793, 1, 0, 0, 0, 795, 798, 1, 0, 0, 0, 796, 794, 1, 0, 0, 0, 
		796, 797, 1, 0, 0, 0, 797, 799, 1, 0, 0, 0, 798, 796, 1, 0, 0, 0, 799, 
		800, 5, 51, 0, 0, 800, 801, 3, 154, 77, 0, 801, 97, 1, 0, 0, 0, 802, 803, 
		3, 152, 76, 0, 803, 804, 5, 41, 0, 0, 804, 805, 3, 100, 50, 0, 805, 99, 
		1, 0, 0, 0, 806, 838, 3, 118, 59, 0, 807, 811, 5, 45, 0, 0, 808, 810, 
		5, 90, 0, 0, 809, 808, 1, 0, 0, 0, 810, 813, 1, 0, 0, 0, 811, 809, 1, 
		0, 0, 0, 811, 812, 1, 0, 0, 0, 812, 828, 1, 0, 0, 0, 813, 811, 1, 0, 0, 
		0, 814, 825, 3, 118, 59, 0, 815, 819, 5, 8, 0, 0, 816, 818, 5, 90, 0, 
		0, 817, 816, 1, 0, 0, 0, 818, 821, 1, 0, 0, 0, 819, 817, 1, 0, 0, 0, 819, 
		820, 1, 0, 0, 0, 820, 822, 1, 0, 0, 0, 821, 819, 1, 0, 0, 0, 822, 824, 
		3, 118, 59, 0, 823, 815, 1, 0, 0, 0, 824, 827, 1, 0, 0, 0, 825, 823, 1, 
		0, 0, 0, 825, 826, 1, 0, 0, 0, 826, 829, 1, 0, 0, 0, 827, 825, 1, 0, 0, 
		0, 828, 814, 1, 0, 0, 0, 828, 829, 1, 0, 0, 0, 829, 833, 1, 0, 0, 0, 830, 
		832, 5, 90, 0, 0, 831, 830, 1, 0, 0, 0, 832, 835, 1, 0, 0, 0, 833, 831, 
		1, 0, 0, 0, 833, 834, 1, 0, 0, 0, 834, 836, 1, 0, 0, 0, 835, 833, 1, 0, 
		0, 0, 836, 838, 5, 46, 0, 0, 837, 806, 1, 0, 0, 0, 837, 807, 1, 0, 0, 
		0, 838, 101, 1, 0, 0, 0, 839, 840, 5, 55, 0, 0, 840, 841, 3, 152, 76, 
		0, 841, 845, 5, 3, 0, 0, 842, 844, 5, 90, 0, 0, 843, 842, 1, 0, 0, 0, 
		844, 847, 1, 0, 0, 0, 845, 843, 1, 0, 0, 0, 845, 846, 1, 0, 0, 0, 846, 
		857, 1, 0, 0, 0, 847, 845, 1, 0, 0, 0, 848, 852, 3, 2, 1, 0, 849, 851, 
		5, 90, 0, 0, 850, 849, 1, 0, 0, 0, 851, 854, 1, 0, 0, 0, 852, 850, 1, 
		0, 0, 0, 852, 853, 1, 0, 0, 0, 853, 856, 1, 0, 0, 0, 854, 852, 1, 0, 0, 
		0, 855, 848, 1, 0, 0, 0, 856, 859, 1, 0, 0, 0, 857, 855, 1, 0, 0, 0, 857, 
		858, 1, 0, 0, 0, 858, 860, 1, 0, 0, 0, 859, 857, 1, 0, 0, 0, 860, 888, 
		5, 4, 0, 0, 861, 863, 5, 90, 0, 0, 862, 861, 1, 0, 0, 0, 863, 866, 1, 
		0, 0, 0, 864, 862, 1, 0, 0, 0, 864, 865, 1, 0, 0, 0, 865, 867, 1, 0, 0, 
		0, 866, 864, 1, 0, 0, 0, 867, 868, 5, 56, 0, 0, 868, 872, 5, 3, 0, 0, 
		869, 871, 5, 90, 0, 0, 870, 869, 1, 0, 0, 0, 871, 874, 1, 0, 0, 0, 872, 
		870, 1, 0, 0, 0, 872, 873, 1, 0, 0, 0, 873, 884, 1, 0, 0, 0, 874, 872, 
		1, 0, 0, 0, 875, 879, 3, 2, 1, 0, 876, 878, 5, 90, 0, 0, 877, 876, 1, 
		0, 0, 0, 878, 881, 1, 0, 0, 0, 879, 877, 1, 0, 0, 0, 879, 880, 1, 0, 0, 
		0, 880, 883, 1, 0, 0, 0, 881, 879, 1, 0, 0, 0, 882, 875, 1, 0, 0, 0, 883, 
		886, 1, 0, 0, 0, 884, 882, 1, 0, 0, 0, 884, 885, 1, 0, 0, 0, 885, 887, 
		1, 0, 0, 0, 886, 884, 1, 0, 0, 0, 887, 889, 5, 4, 0, 0, 888, 864, 1, 0, 
		0, 0, 888, 889, 1, 0, 0, 0, 889, 103, 1, 0, 0, 0, 890, 891, 5, 57, 0, 
		0, 891, 892, 3, 120, 60, 0, 892, 896, 5, 41, 0, 0, 893, 895, 5, 90, 0, 
		0, 894, 893, 1, 0, 0, 0, 895, 898, 1, 0, 0, 0, 896, 894, 1, 0, 0, 0, 896, 
		897, 1, 0, 0, 0, 897, 899, 1, 0, 0, 0, 898, 896, 1, 0, 0, 0, 899, 903, 
		3, 106, 53, 0, 900, 902, 5, 90, 0, 0, 901, 900, 1, 0, 0, 0, 902, 905, 
		1, 0, 0, 0, 903, 901, 1, 0, 0, 0, 903, 904, 1, 0, 0, 0, 904, 906, 1, 0, 
		0, 0, 905, 903, 1, 0, 0, 0, 906, 910, 5, 58, 0, 0, 907, 909, 5, 90, 0, 
		0, 908, 907, 1, 0, 0, 0, 909, 912, 1, 0, 0, 0, 910, 908, 1, 0, 0, 0, 910, 
		911, 1, 0, 0, 0, 911, 913, 1, 0, 0, 0, 912, 910, 1, 0, 0, 0, 913, 914, 
		3, 112, 56, 0, 914, 915, 3, 118, 59, 0, 915, 916, 3, 154, 77, 0, 916, 
		105, 1, 0, 0, 0, 917, 921, 5, 45, 0, 0, 918, 920, 5, 90, 0, 0, 919, 918, 
		1, 0, 0, 0, 920, 923, 1, 0, 0, 0, 921, 919, 1, 0, 0, 0, 921, 922, 1, 0, 
		0, 0, 922, 938, 1, 0, 0, 0, 923, 921, 1, 0, 0, 0, 924, 935, 3, 108, 54, 
		0, 925, 929, 5, 8, 0, 0, 926, 928, 5, 90, 0, 0, 927, 926, 1, 0, 0, 0, 
		928, 931, 1, 0, 0, 0, 929, 927, 1, 0, 0, 0, 929, 930, 1, 0, 0, 0, 930, 
		932, 1, 0, 0, 0, 931, 929, 1, 0, 0, 0, 932, 934, 3, 108, 54, 0, 933, 925, 
		1, 0, 0, 0, 934, 937, 1, 0, 0, 0, 935, 933, 1, 0, 0, 0, 935, 936, 1, 0, 
		0, 0, 936, 939, 1, 0, 0, 0, 937, 935, 1, 0, 0, 0, 938, 924, 1, 0, 0, 0, 
		938, 939, 1, 0, 0, 0, 939, 943, 1, 0, 0, 0, 940, 942, 5, 90, 0, 0, 941, 
		940, 1, 0, 0, 0, 942, 945, 1, 0, 0, 0, 943, 941, 1, 0, 0, 0, 943, 944, 
		1, 0, 0, 0, 944, 946, 1, 0, 0, 0, 945, 943, 1, 0, 0, 0, 946, 947, 5, 46, 
		0, 0, 947, 107, 1, 0, 0, 0, 948, 949, 3, 110, 55, 0, 949, 960, 3, 118, 
		59, 0, 950, 954, 5, 8, 0, 0, 951, 953, 5, 90, 0, 0, 952, 951, 1, 0, 0, 
		0, 953, 956, 1, 0, 0, 0, 954, 952, 1, 0, 0, 0, 954, 955, 1, 0, 0, 0, 955, 
		957, 1, 0, 0, 0, 956, 954, 1, 0, 0, 0, 957, 959, 3, 118, 59, 0, 958, 950, 
		1, 0, 0, 0, 959, 962, 1, 0, 0, 0, 960, 958, 1, 0, 0, 0, 960, 961, 1, 0, 
		0, 0, 961, 109, 1, 0, 0, 0, 962, 960, 1, 0, 0, 0, 963, 964, 7, 1, 0, 0, 
		964, 111, 1, 0, 0, 0, 965, 966, 7, 2, 0, 0, 966, 113, 1, 0, 0, 0, 967, 
		968, 5, 59, 0, 0, 968, 969, 3, 120, 60, 0, 969, 970, 5, 36, 0, 0, 970, 
		971, 5, 87, 0, 0, 971, 972, 3, 154, 77, 0, 972, 997, 1, 0, 0, 0, 973, 
		974, 5, 60, 0, 0, 974, 975, 3, 120, 60, 0, 975, 976, 3, 150, 75, 0, 976, 
		977, 5, 36, 0, 0, 977, 978, 5, 87, 0, 0, 978, 979, 3, 154, 77, 0, 979, 
		997, 1, 0, 0, 0, 980, 981, 5, 61, 0, 0, 981, 982, 3, 120, 60, 0, 982, 
		983, 5, 34, 0, 0, 983, 984, 3, 120, 60, 0, 984, 985, 5, 36, 0, 0, 985, 
		986, 5, 87, 0, 0, 986, 987, 3, 154, 77, 0, 987, 997, 1, 0, 0, 0, 988, 
		989, 5, 62, 0, 0, 989, 990, 3, 120, 60, 0, 990, 991, 5, 63, 0, 0, 991, 
		992, 3, 120, 60, 0, 992, 993, 5, 36, 0, 0, 993, 994, 5, 87, 0, 0, 994, 
		995, 3, 154, 77, 0, 995, 997, 1, 0, 0, 0, 996, 967, 1, 0, 0, 0, 996, 973, 
		1, 0, 0, 0, 996, 980, 1, 0, 0, 0, 996, 988, 1, 0, 0, 0, 997, 115, 1, 0, 
		0, 0, 998, 1002, 5, 45, 0, 0, 999, 1001, 5, 90, 0, 0, 1000, 999, 1, 0, 
		0, 0, 1001, 1004, 1, 0, 0, 0, 1002, 1000, 1, 0, 0, 0, 1002, 1003, 1, 0, 
		0, 0, 1003, 1005, 1, 0, 0, 0, 1004, 1002, 1, 0, 0, 0, 1005, 1016, 3, 118, 
		59, 0, 1006, 1010, 5, 8, 0, 0, 1007, 1009, 5, 90, 0, 0, 1008, 1007, 1, 
		0, 0, 0, 1009, 1012, 1, 0, 0, 0, 1010, 1008, 1, 0, 0, 0, 1010, 1011, 1, 
		0, 0, 0, 1011, 1013, 1, 0, 0, 0, 1012, 1010, 1, 0, 0, 0, 1013, 1015, 3, 
		118, 59, 0, 1014, 1006, 1, 0, 0, 0, 1015, 1018, 1, 0, 0, 0, 1016, 1014, 
		1, 0, 0, 0, 1016, 1017, 1, 0, 0, 0, 1017, 1022, 1, 0, 0, 0, 1018, 1016, 
		1, 0, 0, 0, 1019, 1021, 5, 90, 0, 0, 1020, 1019, 1, 0, 0, 0, 1021, 1024, 
		1, 0, 0, 0, 1022, 1020, 1, 0, 0, 0, 1022, 1023, 1, 0, 0, 0, 1023, 1025, 
		1, 0, 0, 0, 1024, 1022, 1, 0, 0, 0, 1025, 1026, 5, 46, 0, 0, 1026, 117, 
		1, 0, 0, 0, 1027, 1039, 3, 120, 60, 0, 1028, 1029, 5, 64, 0, 0, 1029, 
		1030, 5, 50, 0, 0, 1030, 1031, 3, 120, 60, 0, 1031, 1032, 5, 51, 0, 0, 
		1032, 1039, 1, 0, 0, 0, 1033, 1034, 5, 65, 0, 0, 1034, 1035, 5, 50, 0, 
		0, 1035, 1036, 3, 120, 60, 0, 1036, 1037, 5, 51, 0, 0, 1037, 1039, 1, 
		0, 0, 0, 1038, 1027, 1, 0, 0, 0, 1038, 1028, 1, 0, 0, 0, 1038, 1033, 1, 
		0, 0, 0, 1039, 119, 1, 0, 0, 0, 1040, 1045, 3, 152, 76, 0, 1041, 1042, 
		5, 66, 0, 0, 1042, 1044, 3, 152, 76, 0, 1043, 1041, 1, 0, 0, 0, 1044, 
		1047, 1, 0, 0, 0, 1045, 1043, 1, 0, 0, 0, 1045, 1046, 1, 0, 0, 0, 1046, 
		121, 1, 0, 0, 0, 1047, 1045, 1, 0, 0, 0, 1048, 1049, 5, 67, 0, 0, 1049, 
		1050, 3, 124, 62, 0, 1050, 1051, 3, 154, 77, 0, 1051, 123, 1, 0, 0, 0, 
		1052, 1053, 3, 126, 63, 0, 1053, 125, 1, 0, 0, 0, 1054, 1059, 3, 128, 
		64, 0, 1055, 1056, 5, 68, 0, 0, 1056, 1058, 3, 128, 64, 0, 1057, 1055, 
		1, 0, 0, 0, 1058, 1061, 1, 0, 0, 0, 1059, 1057, 1, 0, 0, 0, 1059, 1060, 
		1, 0, 0, 0, 1060, 127, 1, 0, 0, 0, 1061, 1059, 1, 0, 0, 0, 1062, 1067, 
		3, 130, 65, 0, 1063, 1064, 5, 69, 0, 0, 1064, 1066, 3, 130, 65, 0, 1065, 
		1063, 1, 0, 0, 0, 1066, 1069, 1, 0, 0, 0, 1067, 1065, 1, 0, 0, 0, 1067, 
		1068, 1, 0, 0, 0, 1068, 129, 1, 0, 0, 0, 1069, 1067, 1, 0, 0, 0, 1070, 
		1074, 3, 132, 66, 0, 1071, 1072, 3, 140, 70, 0, 1072, 1073, 3, 132, 66, 
		0, 1073, 1075, 1, 0, 0, 0, 1074, 1071, 1, 0, 0, 0, 1074, 1075, 1, 0, 0, 
		0, 1075, 131, 1, 0, 0, 0, 1076, 1081, 3, 134, 67, 0, 1077, 1078, 7, 3, 
		0, 0, 1078, 1080, 3, 134, 67, 0, 1079, 1077, 1, 0, 0, 0, 1080, 1083, 1, 
		0, 0, 0, 1081, 1079, 1, 0, 0, 0, 1081, 1082, 1, 0, 0, 0, 1082, 133, 1, 
		0, 0, 0, 1083, 1081, 1, 0, 0, 0, 1084, 1089, 3, 136, 68, 0, 1085, 1086, 
		7, 4, 0, 0, 1086, 1088, 3, 136, 68, 0, 1087, 1085, 1, 0, 0, 0, 1088, 1091, 
		1, 0, 0, 0, 1089, 1087, 1, 0, 0, 0, 1089, 1090, 1, 0, 0, 0, 1090, 135, 
		1, 0, 0, 0, 1091, 1089, 1, 0, 0, 0, 1092, 1093, 7, 5, 0, 0, 1093, 1096, 
		3, 136, 68, 0, 1094, 1096, 3, 138, 69, 0, 1095, 1092, 1, 0, 0, 0, 1095, 
		1094, 1, 0, 0, 0, 1096, 137, 1, 0, 0, 0, 1097, 1098, 3, 120, 60, 0, 1098, 
		1107, 5, 50, 0, 0, 1099, 1104, 3, 124, 62, 0, 1100, 1101, 5, 8, 0, 0, 
		1101, 1103, 3, 124, 62, 0, 1102, 1100, 1, 0, 0, 0, 1103, 1106, 1, 0, 0, 
		0, 1104, 1102, 1, 0, 0, 0, 1104, 1105, 1, 0, 0, 0, 1105, 1108, 1, 0, 0, 
		0, 1106, 1104, 1, 0, 0, 0, 1107, 1099, 1, 0, 0, 0, 1107, 1108, 1, 0, 0, 
		0, 1108, 1109, 1, 0, 0, 0, 1109, 1110, 5, 51, 0, 0, 1110, 1118, 1, 0, 
		0, 0, 1111, 1118, 3, 120, 60, 0, 1112, 1118, 3, 142, 71, 0, 1113, 1114, 
		5, 50, 0, 0, 1114, 1115, 3, 124, 62, 0, 1115, 1116, 5, 51, 0, 0, 1116, 
		1118, 1, 0, 0, 0, 1117, 1097, 1, 0, 0, 0, 1117, 1111, 1, 0, 0, 0, 1117, 
		1112, 1, 0, 0, 0, 1117, 1113, 1, 0, 0, 0, 1118, 139, 1, 0, 0, 0, 1119, 
		1120, 7, 6, 0, 0, 1120, 141, 1, 0, 0, 0, 1121, 1125, 3, 148, 74, 0, 1122, 
		1125, 3, 144, 72, 0, 1123, 1125, 3, 146, 73, 0, 1124, 1121, 1, 0, 0, 0, 
		1124, 1122, 1, 0, 0, 0, 1124, 1123, 1, 0, 0, 0, 1125, 143, 1, 0, 0, 0, 
		1126, 1130, 5, 3, 0, 0, 1127, 1129, 5, 90, 0, 0, 1128, 1127, 1, 0, 0, 
		0, 1129, 1132, 1, 0, 0, 0, 1130, 1128, 1, 0, 0, 0, 1130, 1131, 1, 0, 0, 
		0, 1131, 1181, 1, 0, 0, 0, 1132, 1130, 1, 0, 0, 0, 1133, 1137, 5, 87, 
		0, 0, 1134, 1136, 5, 90, 0, 0, 1135, 1134, 1, 0, 0, 0, 1136, 1139, 1, 
		0, 0, 0, 1137, 1135, 1, 0, 0, 0, 1137, 1138, 1, 0, 0, 0, 1138, 1140, 1, 
		0, 0, 0, 1139, 1137, 1, 0, 0, 0, 1140, 1144, 5, 53, 0, 0, 1141, 1143, 
		5, 90, 0, 0, 1142, 1141, 1, 0, 0, 0, 1143, 1146, 1, 0, 0, 0, 1144, 1142, 
		1, 0, 0, 0, 1144, 1145, 1, 0, 0, 0, 1145, 1147, 1, 0, 0, 0, 1146, 1144, 
		1, 0, 0, 0, 1147, 1178, 3, 142, 71, 0, 1148, 1150, 5, 90, 0, 0, 1149, 
		1148, 1, 0, 0, 0, 1150, 1153, 1, 0, 0, 0, 1151, 1149, 1, 0, 0, 0, 1151, 
		1152, 1, 0, 0, 0, 1152, 1154, 1, 0, 0, 0, 1153, 1151, 1, 0, 0, 0, 1154, 
		1158, 5, 8, 0, 0, 1155, 1157, 5, 90, 0, 0, 1156, 1155, 1, 0, 0, 0, 1157, 
		1160, 1, 0, 0, 0, 1158, 1156, 1, 0, 0, 0, 1158, 1159, 1, 0, 0, 0, 1159, 
		1161, 1, 0, 0, 0, 1160, 1158, 1, 0, 0, 0, 1161, 1165, 5, 87, 0, 0, 1162, 
		1164, 5, 90, 0, 0, 1163, 1162, 1, 0, 0, 0, 1164, 1167, 1, 0, 0, 0, 1165, 
		1163, 1, 0, 0, 0, 1165, 1166, 1, 0, 0, 0, 1166, 1168, 1, 0, 0, 0, 1167, 
		1165, 1, 0, 0, 0, 1168, 1172, 5, 53, 0, 0, 1169, 1171, 5, 90, 0, 0, 1170, 
		1169, 1, 0, 0, 0, 1171, 1174, 1, 0, 0, 0, 1172, 1170, 1, 0, 0, 0, 1172, 
		1173, 1, 0, 0, 0, 1173, 1175, 1, 0, 0, 0, 1174, 1172, 1, 0, 0, 0, 1175, 
		1177, 3, 142, 71, 0, 1176, 1151, 1, 0, 0, 0, 1177, 1180, 1, 0, 0, 0, 1178, 
		1176, 1, 0, 0, 0, 1178, 1179, 1, 0, 0, 0, 1179, 1182, 1, 0, 0, 0, 1180, 
		1178, 1, 0, 0, 0, 1181, 1133, 1, 0, 0, 0, 1181, 1182, 1, 0, 0, 0, 1182, 
		1186, 1, 0, 0, 0, 1183, 1185, 5, 90, 0, 0, 1184, 1183, 1, 0, 0, 0, 1185, 
		1188, 1, 0, 0, 0, 1186, 1184, 1, 0, 0, 0, 1186, 1187, 1, 0, 0, 0, 1187, 
		1189, 1, 0, 0, 0, 1188, 1186, 1, 0, 0, 0, 1189, 1190, 5, 4, 0, 0, 1190, 
		145, 1, 0, 0, 0, 1191, 1195, 5, 45, 0, 0, 1192, 1194, 5, 90, 0, 0, 1193, 
		1192, 1, 0, 0, 0, 1194, 1197, 1, 0, 0, 0, 1195, 1193, 1, 0, 0, 0, 1195, 
		1196, 1, 0, 0, 0, 1196, 1218, 1, 0, 0, 0, 1197, 1195, 1, 0, 0, 0, 1198, 
		1215, 3, 142, 71, 0, 1199, 1201, 5, 90, 0, 0, 1200, 1199, 1, 0, 0, 0, 
		1201, 1204, 1, 0, 0, 0, 1202, 1200, 1, 0, 0, 0, 1202, 1203, 1, 0, 0, 0, 
		1203, 1205, 1, 0, 0, 0, 1204, 1202, 1, 0, 0, 0, 1205, 1209, 5, 8, 0, 0, 
		1206, 1208, 5, 90, 0, 0, 1207, 1206, 1, 0, 0, 0, 1208, 1211, 1, 0, 0, 
		0, 1209, 1207, 1, 0, 0, 0, 1209, 1210, 1, 0, 0, 0, 1210, 1212, 1, 0, 0, 
		0, 1211, 1209, 1, 0, 0, 0, 1212, 1214, 3, 142, 71, 0, 1213, 1202, 1, 0, 
		0, 0, 1214, 1217, 1, 0, 0, 0, 1215, 1213, 1, 0, 0, 0, 1215, 1216, 1, 0, 
		0, 0, 1216, 1219, 1, 0, 0, 0, 1217, 1215, 1, 0, 0, 0, 1218, 1198, 1, 0, 
		0, 0, 1218, 1219, 1, 0, 0, 0, 1219, 1223, 1, 0, 0, 0, 1220, 1222, 5, 90, 
		0, 0, 1221, 1220, 1, 0, 0, 0, 1222, 1225, 1, 0, 0, 0, 1223, 1221, 1, 0, 
		0, 0, 1223, 1224, 1, 0, 0, 0, 1224, 1226, 1, 0, 0, 0, 1225, 1223, 1, 0, 
		0, 0, 1226, 1227, 5, 46, 0, 0, 1227, 147, 1, 0, 0, 0, 1228, 1234, 5, 87, 
		0, 0, 1229, 1234, 3, 150, 75, 0, 1230, 1234, 5, 82, 0, 0, 1231, 1234, 
		5, 83, 0, 0, 1232, 1234, 5, 84, 0, 0, 1233, 1228, 1, 0, 0, 0, 1233, 1229, 
		1, 0, 0, 0, 1233, 1230, 1, 0, 0, 0, 1233, 1231, 1, 0, 0, 0, 1233, 1232, 
		1, 0, 0, 0, 1234, 149, 1, 0, 0, 0, 1235, 1237, 5, 71, 0, 0, 1236, 1235, 
		1, 0, 0, 0, 1236, 1237, 1, 0, 0, 0, 1237, 1238, 1, 0, 0, 0, 1238, 1239, 
		5, 86, 0, 0, 1239, 151, 1, 0, 0, 0, 1240, 1241, 7, 7, 0, 0, 1241, 153, 
		1, 0, 0, 0, 1242, 1244, 5, 90, 0, 0, 1243, 1242, 1, 0, 0, 0, 1244, 1245, 
		1, 0, 0, 0, 1245, 1243, 1, 0, 0, 0, 1245, 1246, 1, 0, 0, 0, 1246, 1249, 
		1, 0, 0, 0, 1247, 1249, 5, 0, 0, 1, 1248, 1243, 1, 0, 0, 0, 1248, 1247, 
		1, 0, 0, 0, 1249, 155, 1, 0, 0, 0, 131, 159, 169, 174, 194, 202, 209, 
		214, 231, 238, 245, 250, 285, 293, 299, 309, 315, 330, 336, 349, 388, 
		394, 404, 428, 434, 444, 475, 481, 489, 496, 502, 509, 520, 552, 562, 
		569, 574, 581, 587, 594, 601, 608, 615, 620, 626, 634, 640, 643, 648, 
		659, 665, 676, 684, 690, 693, 698, 705, 710, 715, 722, 732, 739, 744, 
		748, 755, 764, 774, 782, 788, 791, 796, 811, 819, 825, 828, 833, 837, 
		845, 852, 857, 864, 872, 879, 884, 888, 896, 903, 910, 921, 929, 935, 
		938, 943, 954, 960, 996, 1002, 1010, 1016, 1022, 1038, 1045, 1059, 1067, 
		1074, 1081, 1089, 1095, 1104, 1107, 1117, 1124, 1130, 1137, 1144, 1151, 
		1158, 1165, 1172, 1178, 1181, 1186, 1195, 1202, 1209, 1215, 1218, 1223, 
		1233, 1236, 1245, 1248
	];
}
