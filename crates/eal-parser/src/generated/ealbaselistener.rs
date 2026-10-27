// Generated from grammar/EAL.g4 by ANTLR 4.13.2

use super::ealparser::*;
use antlr4rust::tree::ParseTreeListener;

// A complete Visitor for a parse tree produced by EALParser.

pub trait EALBaseListener<'input>:
    ParseTreeListener<'input, EALParserContextType> {

    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_program(&mut self, _ctx: &ProgramContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_program(&mut self, _ctx: &ProgramContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_declaration(&mut self, _ctx: &DeclarationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_declaration(&mut self, _ctx: &DeclarationContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_moduledecl(&mut self, _ctx: &ModuleDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_moduledecl(&mut self, _ctx: &ModuleDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_importdecl(&mut self, _ctx: &ImportDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_importdecl(&mut self, _ctx: &ImportDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_contextdecl(&mut self, _ctx: &ContextDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_contextdecl(&mut self, _ctx: &ContextDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_contextattribute(&mut self, _ctx: &ContextAttributeContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_contextattribute(&mut self, _ctx: &ContextAttributeContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_environmentdecl(&mut self, _ctx: &EnvironmentDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_environmentdecl(&mut self, _ctx: &EnvironmentDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_tooldecl(&mut self, _ctx: &ToolDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_tooldecl(&mut self, _ctx: &ToolDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_versionfield(&mut self, _ctx: &VersionFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_versionfield(&mut self, _ctx: &VersionFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_evidencedecl(&mut self, _ctx: &EvidenceDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_evidencedecl(&mut self, _ctx: &EvidenceDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_evidencefield(&mut self, _ctx: &EvidenceFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_evidencefield(&mut self, _ctx: &EvidenceFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_toolfield(&mut self, _ctx: &ToolFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_toolfield(&mut self, _ctx: &ToolFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_kindfield(&mut self, _ctx: &KindFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_kindfield(&mut self, _ctx: &KindFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_environmentfield(&mut self, _ctx: &EnvironmentFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_environmentfield(&mut self, _ctx: &EnvironmentFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_maxagefield(&mut self, _ctx: &MaxAgeFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_maxagefield(&mut self, _ctx: &MaxAgeFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_inputfield(&mut self, _ctx: &InputFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_inputfield(&mut self, _ctx: &InputFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_inputbindingfield(&mut self, _ctx: &InputBindingFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_inputbindingfield(&mut self, _ctx: &InputBindingFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_derivefield(&mut self, _ctx: &DeriveFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_derivefield(&mut self, _ctx: &DeriveFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_assumptiondecl(&mut self, _ctx: &AssumptionDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_assumptiondecl(&mut self, _ctx: &AssumptionDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_assumptionfield(&mut self, _ctx: &AssumptionFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_assumptionfield(&mut self, _ctx: &AssumptionFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_statementfield(&mut self, _ctx: &StatementFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_statementfield(&mut self, _ctx: &StatementFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_validatefield(&mut self, _ctx: &ValidateFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_validatefield(&mut self, _ctx: &ValidateFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_validfromfield(&mut self, _ctx: &ValidFromFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_validfromfield(&mut self, _ctx: &ValidFromFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_validuntilfield(&mut self, _ctx: &ValidUntilFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_validuntilfield(&mut self, _ctx: &ValidUntilFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_reasoningdecl(&mut self, _ctx: &ReasoningDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_reasoningdecl(&mut self, _ctx: &ReasoningDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_reasoningfield(&mut self, _ctx: &ReasoningFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_reasoningfield(&mut self, _ctx: &ReasoningFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_methodfield(&mut self, _ctx: &MethodFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_methodfield(&mut self, _ctx: &MethodFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_rationalefield(&mut self, _ctx: &RationaleFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_rationalefield(&mut self, _ctx: &RationaleFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_backingfield(&mut self, _ctx: &BackingFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_backingfield(&mut self, _ctx: &BackingFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_transferfield(&mut self, _ctx: &TransferFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_transferfield(&mut self, _ctx: &TransferFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_claimdecl(&mut self, _ctx: &ClaimDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_claimdecl(&mut self, _ctx: &ClaimDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_claimfield(&mut self, _ctx: &ClaimFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_claimfield(&mut self, _ctx: &ClaimFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_propositiondecl(&mut self, _ctx: &PropositionDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_propositiondecl(&mut self, _ctx: &PropositionDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_propositionfield(&mut self, _ctx: &PropositionFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_propositionfield(&mut self, _ctx: &PropositionFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_subjectfield(&mut self, _ctx: &SubjectFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_subjectfield(&mut self, _ctx: &SubjectFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_quantityfield(&mut self, _ctx: &QuantityFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_quantityfield(&mut self, _ctx: &QuantityFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unitfield(&mut self, _ctx: &UnitFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unitfield(&mut self, _ctx: &UnitFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_scopefield(&mut self, _ctx: &ScopeFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_scopefield(&mut self, _ctx: &ScopeFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_queryfield(&mut self, _ctx: &QueryFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_queryfield(&mut self, _ctx: &QueryFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_resultfield(&mut self, _ctx: &ResultFieldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_resultfield(&mut self, _ctx: &ResultFieldContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_argumentdecl(&mut self, _ctx: &ArgumentDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_argumentdecl(&mut self, _ctx: &ArgumentDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_argumentflow(&mut self, _ctx: &ArgumentFlowContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_argumentflow(&mut self, _ctx: &ArgumentFlowContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_support(&mut self, _ctx: &SupportContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_support(&mut self, _ctx: &SupportContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_supportgroup(&mut self, _ctx: &SupportGroupContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_supportgroup(&mut self, _ctx: &SupportGroupContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_supportkind(&mut self, _ctx: &SupportKindContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_supportkind(&mut self, _ctx: &SupportKindContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patterndecl(&mut self, _ctx: &PatternDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patterndecl(&mut self, _ctx: &PatternDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternparameter(&mut self, _ctx: &PatternParameterContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternparameter(&mut self, _ctx: &PatternParameterContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_parameterkind(&mut self, _ctx: &ParameterKindContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_parameterkind(&mut self, _ctx: &ParameterKindContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_applicationdecl(&mut self, _ctx: &ApplicationDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_applicationdecl(&mut self, _ctx: &ApplicationDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternbinding(&mut self, _ctx: &PatternBindingContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternbinding(&mut self, _ctx: &PatternBindingContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_bindingvalue(&mut self, _ctx: &BindingValueContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_bindingvalue(&mut self, _ctx: &BindingValueContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternguard(&mut self, _ctx: &PatternGuardContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternguard(&mut self, _ctx: &PatternGuardContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_objectiondecl(&mut self, _ctx: &ObjectionDeclContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_objectiondecl(&mut self, _ctx: &ObjectionDeclContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_objectionsupport(&mut self, _ctx: &ObjectionSupportContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_objectionsupport(&mut self, _ctx: &ObjectionSupportContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_objectiongroup(&mut self, _ctx: &ObjectionGroupContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_objectiongroup(&mut self, _ctx: &ObjectionGroupContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_objectionsupportkind(&mut self, _ctx: &ObjectionSupportKindContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_objectionsupportkind(&mut self, _ctx: &ObjectionSupportKindContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_targetkind(&mut self, _ctx: &TargetKindContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_targetkind(&mut self, _ctx: &TargetKindContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_argumentationdirective(&mut self, _ctx: &ArgumentationDirectiveContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_argumentationdirective(&mut self, _ctx: &ArgumentationDirectiveContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_referencelist(&mut self, _ctx: &ReferenceListContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_referencelist(&mut self, _ctx: &ReferenceListContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_reference(&mut self, _ctx: &ReferenceContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_reference(&mut self, _ctx: &ReferenceContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_qualifiedname(&mut self, _ctx: &QualifiedNameContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_qualifiedname(&mut self, _ctx: &QualifiedNameContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_predicate(&mut self, _ctx: &PredicateContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_predicate(&mut self, _ctx: &PredicateContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_expression(&mut self, _ctx: &ExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_expression(&mut self, _ctx: &ExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_orexpression(&mut self, _ctx: &OrExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_orexpression(&mut self, _ctx: &OrExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_andexpression(&mut self, _ctx: &AndExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_andexpression(&mut self, _ctx: &AndExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_comparisonexpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_comparisonexpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_additiveexpression(&mut self, _ctx: &AdditiveExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_additiveexpression(&mut self, _ctx: &AdditiveExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_multiplicativeexpression(&mut self, _ctx: &MultiplicativeExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_multiplicativeexpression(&mut self, _ctx: &MultiplicativeExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unaryexpression(&mut self, _ctx: &UnaryExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unaryexpression(&mut self, _ctx: &UnaryExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_primaryexpression(&mut self, _ctx: &PrimaryExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_primaryexpression(&mut self, _ctx: &PrimaryExpressionContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_comparator(&mut self, _ctx: &ComparatorContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_comparator(&mut self, _ctx: &ComparatorContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonvalue(&mut self, _ctx: &JsonValueContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonvalue(&mut self, _ctx: &JsonValueContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonobject(&mut self, _ctx: &JsonObjectContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonobject(&mut self, _ctx: &JsonObjectContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonarray(&mut self, _ctx: &JsonArrayContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonarray(&mut self, _ctx: &JsonArrayContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_jsonscalar(&mut self, _ctx: &JsonScalarContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_jsonscalar(&mut self, _ctx: &JsonScalarContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_signednumber(&mut self, _ctx: &SignedNumberContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_signednumber(&mut self, _ctx: &SignedNumberContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_identifier(&mut self, _ctx: &IdentifierContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_identifier(&mut self, _ctx: &IdentifierContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_lineend(&mut self, _ctx: &LineEndContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  EALBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_lineend(&mut self, _ctx: &LineEndContext<'input>) {}


}