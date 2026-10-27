#![allow(nonstandard_style)]
// Generated from grammar/EAL.g4 by ANTLR 4.13.2
use antlr4rust::tree::{ParseTreeVisitor,ParseTreeVisitorCompat};
use super::ealparser::*;

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link EALParser}.
 */
pub trait EALVisitor<'input>: ParseTreeVisitor<'input,EALParserContextType>{
	/**
	 * Visit a parse tree produced by {@link EALParser#program}.
	 * @param ctx the parse tree
	 */
	fn visit_program(&mut self, ctx: &ProgramContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#declaration}.
	 * @param ctx the parse tree
	 */
	fn visit_declaration(&mut self, ctx: &DeclarationContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#moduleDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_moduleDecl(&mut self, ctx: &ModuleDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#importDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_importDecl(&mut self, ctx: &ImportDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#contextDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_contextDecl(&mut self, ctx: &ContextDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#contextAttribute}.
	 * @param ctx the parse tree
	 */
	fn visit_contextAttribute(&mut self, ctx: &ContextAttributeContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#environmentDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_environmentDecl(&mut self, ctx: &EnvironmentDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#toolDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_toolDecl(&mut self, ctx: &ToolDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#versionField}.
	 * @param ctx the parse tree
	 */
	fn visit_versionField(&mut self, ctx: &VersionFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#evidenceDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_evidenceDecl(&mut self, ctx: &EvidenceDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#evidenceField}.
	 * @param ctx the parse tree
	 */
	fn visit_evidenceField(&mut self, ctx: &EvidenceFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#toolField}.
	 * @param ctx the parse tree
	 */
	fn visit_toolField(&mut self, ctx: &ToolFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#kindField}.
	 * @param ctx the parse tree
	 */
	fn visit_kindField(&mut self, ctx: &KindFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#environmentField}.
	 * @param ctx the parse tree
	 */
	fn visit_environmentField(&mut self, ctx: &EnvironmentFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#maxAgeField}.
	 * @param ctx the parse tree
	 */
	fn visit_maxAgeField(&mut self, ctx: &MaxAgeFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#inputField}.
	 * @param ctx the parse tree
	 */
	fn visit_inputField(&mut self, ctx: &InputFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#inputBindingField}.
	 * @param ctx the parse tree
	 */
	fn visit_inputBindingField(&mut self, ctx: &InputBindingFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#deriveField}.
	 * @param ctx the parse tree
	 */
	fn visit_deriveField(&mut self, ctx: &DeriveFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#assumptionDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_assumptionDecl(&mut self, ctx: &AssumptionDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#assumptionField}.
	 * @param ctx the parse tree
	 */
	fn visit_assumptionField(&mut self, ctx: &AssumptionFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#statementField}.
	 * @param ctx the parse tree
	 */
	fn visit_statementField(&mut self, ctx: &StatementFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#validateField}.
	 * @param ctx the parse tree
	 */
	fn visit_validateField(&mut self, ctx: &ValidateFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#validFromField}.
	 * @param ctx the parse tree
	 */
	fn visit_validFromField(&mut self, ctx: &ValidFromFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#validUntilField}.
	 * @param ctx the parse tree
	 */
	fn visit_validUntilField(&mut self, ctx: &ValidUntilFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#reasoningDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_reasoningDecl(&mut self, ctx: &ReasoningDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#reasoningField}.
	 * @param ctx the parse tree
	 */
	fn visit_reasoningField(&mut self, ctx: &ReasoningFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#methodField}.
	 * @param ctx the parse tree
	 */
	fn visit_methodField(&mut self, ctx: &MethodFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#rationaleField}.
	 * @param ctx the parse tree
	 */
	fn visit_rationaleField(&mut self, ctx: &RationaleFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#backingField}.
	 * @param ctx the parse tree
	 */
	fn visit_backingField(&mut self, ctx: &BackingFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#transferField}.
	 * @param ctx the parse tree
	 */
	fn visit_transferField(&mut self, ctx: &TransferFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#claimDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_claimDecl(&mut self, ctx: &ClaimDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#claimField}.
	 * @param ctx the parse tree
	 */
	fn visit_claimField(&mut self, ctx: &ClaimFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#propositionDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_propositionDecl(&mut self, ctx: &PropositionDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#propositionField}.
	 * @param ctx the parse tree
	 */
	fn visit_propositionField(&mut self, ctx: &PropositionFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#subjectField}.
	 * @param ctx the parse tree
	 */
	fn visit_subjectField(&mut self, ctx: &SubjectFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#quantityField}.
	 * @param ctx the parse tree
	 */
	fn visit_quantityField(&mut self, ctx: &QuantityFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#unitField}.
	 * @param ctx the parse tree
	 */
	fn visit_unitField(&mut self, ctx: &UnitFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#scopeField}.
	 * @param ctx the parse tree
	 */
	fn visit_scopeField(&mut self, ctx: &ScopeFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#queryField}.
	 * @param ctx the parse tree
	 */
	fn visit_queryField(&mut self, ctx: &QueryFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#resultField}.
	 * @param ctx the parse tree
	 */
	fn visit_resultField(&mut self, ctx: &ResultFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_argumentDecl(&mut self, ctx: &ArgumentDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentFlow}.
	 * @param ctx the parse tree
	 */
	fn visit_argumentFlow(&mut self, ctx: &ArgumentFlowContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#support}.
	 * @param ctx the parse tree
	 */
	fn visit_support(&mut self, ctx: &SupportContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#supportGroup}.
	 * @param ctx the parse tree
	 */
	fn visit_supportGroup(&mut self, ctx: &SupportGroupContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#supportKind}.
	 * @param ctx the parse tree
	 */
	fn visit_supportKind(&mut self, ctx: &SupportKindContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#patternDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_patternDecl(&mut self, ctx: &PatternDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#patternParameter}.
	 * @param ctx the parse tree
	 */
	fn visit_patternParameter(&mut self, ctx: &PatternParameterContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#parameterKind}.
	 * @param ctx the parse tree
	 */
	fn visit_parameterKind(&mut self, ctx: &ParameterKindContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#applicationDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_applicationDecl(&mut self, ctx: &ApplicationDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#patternBinding}.
	 * @param ctx the parse tree
	 */
	fn visit_patternBinding(&mut self, ctx: &PatternBindingContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#bindingValue}.
	 * @param ctx the parse tree
	 */
	fn visit_bindingValue(&mut self, ctx: &BindingValueContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#patternGuard}.
	 * @param ctx the parse tree
	 */
	fn visit_patternGuard(&mut self, ctx: &PatternGuardContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionDecl}.
	 * @param ctx the parse tree
	 */
	fn visit_objectionDecl(&mut self, ctx: &ObjectionDeclContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionSupport}.
	 * @param ctx the parse tree
	 */
	fn visit_objectionSupport(&mut self, ctx: &ObjectionSupportContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionGroup}.
	 * @param ctx the parse tree
	 */
	fn visit_objectionGroup(&mut self, ctx: &ObjectionGroupContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionSupportKind}.
	 * @param ctx the parse tree
	 */
	fn visit_objectionSupportKind(&mut self, ctx: &ObjectionSupportKindContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#targetKind}.
	 * @param ctx the parse tree
	 */
	fn visit_targetKind(&mut self, ctx: &TargetKindContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentationDirective}.
	 * @param ctx the parse tree
	 */
	fn visit_argumentationDirective(&mut self, ctx: &ArgumentationDirectiveContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#referenceList}.
	 * @param ctx the parse tree
	 */
	fn visit_referenceList(&mut self, ctx: &ReferenceListContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#reference}.
	 * @param ctx the parse tree
	 */
	fn visit_reference(&mut self, ctx: &ReferenceContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#qualifiedName}.
	 * @param ctx the parse tree
	 */
	fn visit_qualifiedName(&mut self, ctx: &QualifiedNameContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#predicate}.
	 * @param ctx the parse tree
	 */
	fn visit_predicate(&mut self, ctx: &PredicateContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#expression}.
	 * @param ctx the parse tree
	 */
	fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#orExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_orExpression(&mut self, ctx: &OrExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#andExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#comparisonExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_comparisonExpression(&mut self, ctx: &ComparisonExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#additiveExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_additiveExpression(&mut self, ctx: &AdditiveExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#multiplicativeExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_multiplicativeExpression(&mut self, ctx: &MultiplicativeExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#unaryExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_unaryExpression(&mut self, ctx: &UnaryExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#primaryExpression}.
	 * @param ctx the parse tree
	 */
	fn visit_primaryExpression(&mut self, ctx: &PrimaryExpressionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#comparator}.
	 * @param ctx the parse tree
	 */
	fn visit_comparator(&mut self, ctx: &ComparatorContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonValue}.
	 * @param ctx the parse tree
	 */
	fn visit_jsonValue(&mut self, ctx: &JsonValueContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonObject}.
	 * @param ctx the parse tree
	 */
	fn visit_jsonObject(&mut self, ctx: &JsonObjectContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonArray}.
	 * @param ctx the parse tree
	 */
	fn visit_jsonArray(&mut self, ctx: &JsonArrayContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonScalar}.
	 * @param ctx the parse tree
	 */
	fn visit_jsonScalar(&mut self, ctx: &JsonScalarContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#signedNumber}.
	 * @param ctx the parse tree
	 */
	fn visit_signedNumber(&mut self, ctx: &SignedNumberContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#identifier}.
	 * @param ctx the parse tree
	 */
	fn visit_identifier(&mut self, ctx: &IdentifierContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link EALParser#lineEnd}.
	 * @param ctx the parse tree
	 */
	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) { self.visit_children(ctx) }

}

pub trait EALVisitorCompat<'input>:ParseTreeVisitorCompat<'input, Node= EALParserContextType>{
	/**
	 * Visit a parse tree produced by {@link EALParser#program}.
	 * @param ctx the parse tree
	 */
		fn visit_program(&mut self, ctx: &ProgramContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#declaration}.
	 * @param ctx the parse tree
	 */
		fn visit_declaration(&mut self, ctx: &DeclarationContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#moduleDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_moduleDecl(&mut self, ctx: &ModuleDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#importDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_importDecl(&mut self, ctx: &ImportDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#contextDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_contextDecl(&mut self, ctx: &ContextDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#contextAttribute}.
	 * @param ctx the parse tree
	 */
		fn visit_contextAttribute(&mut self, ctx: &ContextAttributeContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#environmentDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_environmentDecl(&mut self, ctx: &EnvironmentDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#toolDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_toolDecl(&mut self, ctx: &ToolDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#versionField}.
	 * @param ctx the parse tree
	 */
		fn visit_versionField(&mut self, ctx: &VersionFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#evidenceDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_evidenceDecl(&mut self, ctx: &EvidenceDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#evidenceField}.
	 * @param ctx the parse tree
	 */
		fn visit_evidenceField(&mut self, ctx: &EvidenceFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#toolField}.
	 * @param ctx the parse tree
	 */
		fn visit_toolField(&mut self, ctx: &ToolFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#kindField}.
	 * @param ctx the parse tree
	 */
		fn visit_kindField(&mut self, ctx: &KindFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#environmentField}.
	 * @param ctx the parse tree
	 */
		fn visit_environmentField(&mut self, ctx: &EnvironmentFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#maxAgeField}.
	 * @param ctx the parse tree
	 */
		fn visit_maxAgeField(&mut self, ctx: &MaxAgeFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#inputField}.
	 * @param ctx the parse tree
	 */
		fn visit_inputField(&mut self, ctx: &InputFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#inputBindingField}.
	 * @param ctx the parse tree
	 */
		fn visit_inputBindingField(&mut self, ctx: &InputBindingFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#deriveField}.
	 * @param ctx the parse tree
	 */
		fn visit_deriveField(&mut self, ctx: &DeriveFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#assumptionDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_assumptionDecl(&mut self, ctx: &AssumptionDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#assumptionField}.
	 * @param ctx the parse tree
	 */
		fn visit_assumptionField(&mut self, ctx: &AssumptionFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#statementField}.
	 * @param ctx the parse tree
	 */
		fn visit_statementField(&mut self, ctx: &StatementFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#validateField}.
	 * @param ctx the parse tree
	 */
		fn visit_validateField(&mut self, ctx: &ValidateFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#validFromField}.
	 * @param ctx the parse tree
	 */
		fn visit_validFromField(&mut self, ctx: &ValidFromFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#validUntilField}.
	 * @param ctx the parse tree
	 */
		fn visit_validUntilField(&mut self, ctx: &ValidUntilFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#reasoningDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_reasoningDecl(&mut self, ctx: &ReasoningDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#reasoningField}.
	 * @param ctx the parse tree
	 */
		fn visit_reasoningField(&mut self, ctx: &ReasoningFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#methodField}.
	 * @param ctx the parse tree
	 */
		fn visit_methodField(&mut self, ctx: &MethodFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#rationaleField}.
	 * @param ctx the parse tree
	 */
		fn visit_rationaleField(&mut self, ctx: &RationaleFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#backingField}.
	 * @param ctx the parse tree
	 */
		fn visit_backingField(&mut self, ctx: &BackingFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#transferField}.
	 * @param ctx the parse tree
	 */
		fn visit_transferField(&mut self, ctx: &TransferFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#claimDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_claimDecl(&mut self, ctx: &ClaimDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#claimField}.
	 * @param ctx the parse tree
	 */
		fn visit_claimField(&mut self, ctx: &ClaimFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#propositionDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_propositionDecl(&mut self, ctx: &PropositionDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#propositionField}.
	 * @param ctx the parse tree
	 */
		fn visit_propositionField(&mut self, ctx: &PropositionFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#subjectField}.
	 * @param ctx the parse tree
	 */
		fn visit_subjectField(&mut self, ctx: &SubjectFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#quantityField}.
	 * @param ctx the parse tree
	 */
		fn visit_quantityField(&mut self, ctx: &QuantityFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#unitField}.
	 * @param ctx the parse tree
	 */
		fn visit_unitField(&mut self, ctx: &UnitFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#scopeField}.
	 * @param ctx the parse tree
	 */
		fn visit_scopeField(&mut self, ctx: &ScopeFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#queryField}.
	 * @param ctx the parse tree
	 */
		fn visit_queryField(&mut self, ctx: &QueryFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#resultField}.
	 * @param ctx the parse tree
	 */
		fn visit_resultField(&mut self, ctx: &ResultFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_argumentDecl(&mut self, ctx: &ArgumentDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentFlow}.
	 * @param ctx the parse tree
	 */
		fn visit_argumentFlow(&mut self, ctx: &ArgumentFlowContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#support}.
	 * @param ctx the parse tree
	 */
		fn visit_support(&mut self, ctx: &SupportContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#supportGroup}.
	 * @param ctx the parse tree
	 */
		fn visit_supportGroup(&mut self, ctx: &SupportGroupContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#supportKind}.
	 * @param ctx the parse tree
	 */
		fn visit_supportKind(&mut self, ctx: &SupportKindContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#patternDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_patternDecl(&mut self, ctx: &PatternDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#patternParameter}.
	 * @param ctx the parse tree
	 */
		fn visit_patternParameter(&mut self, ctx: &PatternParameterContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#parameterKind}.
	 * @param ctx the parse tree
	 */
		fn visit_parameterKind(&mut self, ctx: &ParameterKindContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#applicationDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_applicationDecl(&mut self, ctx: &ApplicationDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#patternBinding}.
	 * @param ctx the parse tree
	 */
		fn visit_patternBinding(&mut self, ctx: &PatternBindingContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#bindingValue}.
	 * @param ctx the parse tree
	 */
		fn visit_bindingValue(&mut self, ctx: &BindingValueContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#patternGuard}.
	 * @param ctx the parse tree
	 */
		fn visit_patternGuard(&mut self, ctx: &PatternGuardContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionDecl}.
	 * @param ctx the parse tree
	 */
		fn visit_objectionDecl(&mut self, ctx: &ObjectionDeclContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionSupport}.
	 * @param ctx the parse tree
	 */
		fn visit_objectionSupport(&mut self, ctx: &ObjectionSupportContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionGroup}.
	 * @param ctx the parse tree
	 */
		fn visit_objectionGroup(&mut self, ctx: &ObjectionGroupContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#objectionSupportKind}.
	 * @param ctx the parse tree
	 */
		fn visit_objectionSupportKind(&mut self, ctx: &ObjectionSupportKindContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#targetKind}.
	 * @param ctx the parse tree
	 */
		fn visit_targetKind(&mut self, ctx: &TargetKindContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#argumentationDirective}.
	 * @param ctx the parse tree
	 */
		fn visit_argumentationDirective(&mut self, ctx: &ArgumentationDirectiveContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#referenceList}.
	 * @param ctx the parse tree
	 */
		fn visit_referenceList(&mut self, ctx: &ReferenceListContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#reference}.
	 * @param ctx the parse tree
	 */
		fn visit_reference(&mut self, ctx: &ReferenceContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#qualifiedName}.
	 * @param ctx the parse tree
	 */
		fn visit_qualifiedName(&mut self, ctx: &QualifiedNameContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#predicate}.
	 * @param ctx the parse tree
	 */
		fn visit_predicate(&mut self, ctx: &PredicateContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#expression}.
	 * @param ctx the parse tree
	 */
		fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#orExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_orExpression(&mut self, ctx: &OrExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#andExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#comparisonExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_comparisonExpression(&mut self, ctx: &ComparisonExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#additiveExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_additiveExpression(&mut self, ctx: &AdditiveExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#multiplicativeExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_multiplicativeExpression(&mut self, ctx: &MultiplicativeExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#unaryExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_unaryExpression(&mut self, ctx: &UnaryExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#primaryExpression}.
	 * @param ctx the parse tree
	 */
		fn visit_primaryExpression(&mut self, ctx: &PrimaryExpressionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#comparator}.
	 * @param ctx the parse tree
	 */
		fn visit_comparator(&mut self, ctx: &ComparatorContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonValue}.
	 * @param ctx the parse tree
	 */
		fn visit_jsonValue(&mut self, ctx: &JsonValueContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonObject}.
	 * @param ctx the parse tree
	 */
		fn visit_jsonObject(&mut self, ctx: &JsonObjectContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonArray}.
	 * @param ctx the parse tree
	 */
		fn visit_jsonArray(&mut self, ctx: &JsonArrayContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#jsonScalar}.
	 * @param ctx the parse tree
	 */
		fn visit_jsonScalar(&mut self, ctx: &JsonScalarContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#signedNumber}.
	 * @param ctx the parse tree
	 */
		fn visit_signedNumber(&mut self, ctx: &SignedNumberContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#identifier}.
	 * @param ctx the parse tree
	 */
		fn visit_identifier(&mut self, ctx: &IdentifierContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link EALParser#lineEnd}.
	 * @param ctx the parse tree
	 */
		fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

}

impl<'input,T> EALVisitor<'input> for T
where
	T: EALVisitorCompat<'input>
{
	fn visit_program(&mut self, ctx: &ProgramContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_program(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_declaration(&mut self, ctx: &DeclarationContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_declaration(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_moduleDecl(&mut self, ctx: &ModuleDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_moduleDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_importDecl(&mut self, ctx: &ImportDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_importDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_contextDecl(&mut self, ctx: &ContextDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_contextDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_contextAttribute(&mut self, ctx: &ContextAttributeContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_contextAttribute(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_environmentDecl(&mut self, ctx: &EnvironmentDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_environmentDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_toolDecl(&mut self, ctx: &ToolDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_toolDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_versionField(&mut self, ctx: &VersionFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_versionField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_evidenceDecl(&mut self, ctx: &EvidenceDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_evidenceDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_evidenceField(&mut self, ctx: &EvidenceFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_evidenceField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_toolField(&mut self, ctx: &ToolFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_toolField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_kindField(&mut self, ctx: &KindFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_kindField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_environmentField(&mut self, ctx: &EnvironmentFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_environmentField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_maxAgeField(&mut self, ctx: &MaxAgeFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_maxAgeField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_inputField(&mut self, ctx: &InputFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_inputField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_inputBindingField(&mut self, ctx: &InputBindingFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_inputBindingField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_deriveField(&mut self, ctx: &DeriveFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_deriveField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_assumptionDecl(&mut self, ctx: &AssumptionDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_assumptionDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_assumptionField(&mut self, ctx: &AssumptionFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_assumptionField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_statementField(&mut self, ctx: &StatementFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_statementField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_validateField(&mut self, ctx: &ValidateFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_validateField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_validFromField(&mut self, ctx: &ValidFromFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_validFromField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_validUntilField(&mut self, ctx: &ValidUntilFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_validUntilField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_reasoningDecl(&mut self, ctx: &ReasoningDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_reasoningDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_reasoningField(&mut self, ctx: &ReasoningFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_reasoningField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_methodField(&mut self, ctx: &MethodFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_methodField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_rationaleField(&mut self, ctx: &RationaleFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_rationaleField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_backingField(&mut self, ctx: &BackingFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_backingField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_transferField(&mut self, ctx: &TransferFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_transferField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_claimDecl(&mut self, ctx: &ClaimDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_claimDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_claimField(&mut self, ctx: &ClaimFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_claimField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_propositionDecl(&mut self, ctx: &PropositionDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_propositionDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_propositionField(&mut self, ctx: &PropositionFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_propositionField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_subjectField(&mut self, ctx: &SubjectFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_subjectField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_quantityField(&mut self, ctx: &QuantityFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_quantityField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_unitField(&mut self, ctx: &UnitFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_unitField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_scopeField(&mut self, ctx: &ScopeFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_scopeField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_queryField(&mut self, ctx: &QueryFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_queryField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_resultField(&mut self, ctx: &ResultFieldContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_resultField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_argumentDecl(&mut self, ctx: &ArgumentDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_argumentDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_argumentFlow(&mut self, ctx: &ArgumentFlowContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_argumentFlow(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_support(&mut self, ctx: &SupportContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_support(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_supportGroup(&mut self, ctx: &SupportGroupContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_supportGroup(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_supportKind(&mut self, ctx: &SupportKindContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_supportKind(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_patternDecl(&mut self, ctx: &PatternDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_patternDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_patternParameter(&mut self, ctx: &PatternParameterContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_patternParameter(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_parameterKind(&mut self, ctx: &ParameterKindContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_parameterKind(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_applicationDecl(&mut self, ctx: &ApplicationDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_applicationDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_patternBinding(&mut self, ctx: &PatternBindingContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_patternBinding(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_bindingValue(&mut self, ctx: &BindingValueContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_bindingValue(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_patternGuard(&mut self, ctx: &PatternGuardContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_patternGuard(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_objectionDecl(&mut self, ctx: &ObjectionDeclContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_objectionDecl(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_objectionSupport(&mut self, ctx: &ObjectionSupportContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_objectionSupport(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_objectionGroup(&mut self, ctx: &ObjectionGroupContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_objectionGroup(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_objectionSupportKind(&mut self, ctx: &ObjectionSupportKindContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_objectionSupportKind(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_targetKind(&mut self, ctx: &TargetKindContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_targetKind(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_argumentationDirective(&mut self, ctx: &ArgumentationDirectiveContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_argumentationDirective(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_referenceList(&mut self, ctx: &ReferenceListContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_referenceList(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_reference(&mut self, ctx: &ReferenceContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_reference(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_qualifiedName(&mut self, ctx: &QualifiedNameContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_qualifiedName(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_predicate(&mut self, ctx: &PredicateContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_predicate(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_expression(&mut self, ctx: &ExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_expression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_orExpression(&mut self, ctx: &OrExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_orExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_andExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_comparisonExpression(&mut self, ctx: &ComparisonExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_comparisonExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_additiveExpression(&mut self, ctx: &AdditiveExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_additiveExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_multiplicativeExpression(&mut self, ctx: &MultiplicativeExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_multiplicativeExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_unaryExpression(&mut self, ctx: &UnaryExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_unaryExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_primaryExpression(&mut self, ctx: &PrimaryExpressionContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_primaryExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_comparator(&mut self, ctx: &ComparatorContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_comparator(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_jsonValue(&mut self, ctx: &JsonValueContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_jsonValue(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_jsonObject(&mut self, ctx: &JsonObjectContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_jsonObject(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_jsonArray(&mut self, ctx: &JsonArrayContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_jsonArray(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_jsonScalar(&mut self, ctx: &JsonScalarContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_jsonScalar(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_signedNumber(&mut self, ctx: &SignedNumberContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_signedNumber(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_identifier(&mut self, ctx: &IdentifierContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_identifier(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>){
		let result = <Self as EALVisitorCompat>::visit_lineEnd(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

}